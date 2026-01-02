"""Semantic embedding computation for video frames."""

from dataclasses import dataclass
from typing import Sequence

import numpy as np
import torch
import torchvision.models as models
import torchvision.transforms as transforms
from numpy.typing import NDArray
from PIL import Image

from scenesplit.constants import QualityPreset
from scenesplit.video import Frame


@dataclass(frozen=True)
class EmbeddedFrame:
    """A frame with its computed embedding vector."""

    frame: Frame
    embedding: NDArray[np.float32]

    @property
    def index(self) -> int:
        """Original frame index."""
        return self.frame.index

    @property
    def timestamp_seconds(self) -> float:
        """Frame timestamp in seconds."""
        return self.frame.timestamp_seconds


class EmbeddingModel:
    """Compute semantic embeddings for video frames using a pretrained vision model.

    Uses ResNet50 pretrained on ImageNet to extract feature vectors.
    All computation is done locally without network access.
    """

    def __init__(self, quality: QualityPreset = QualityPreset.BALANCED) -> None:
        """Initialize the embedding model.

        Args:
            quality: Quality preset affecting batch size and image scaling.
        """
        self.quality = quality
        self.device = self._get_device()

        # Load pretrained ResNet50 and remove the final classification layer
        # to get 2048-dimensional feature vectors
        self._model = models.resnet50(weights=models.ResNet50_Weights.IMAGENET1K_V2)
        self._model = torch.nn.Sequential(*list(self._model.children())[:-1])
        self._model = self._model.to(self.device)
        self._model.eval()

        # Preprocessing transform for ImageNet-pretrained models
        self._transform = transforms.Compose([
            transforms.Resize(256),
            transforms.CenterCrop(224),
            transforms.ToTensor(),
            transforms.Normalize(
                mean=[0.485, 0.456, 0.406],
                std=[0.229, 0.224, 0.225],
            ),
        ])

    def _get_device(self) -> torch.device:
        """Get the best available compute device."""
        if torch.cuda.is_available():
            return torch.device("cuda")
        elif torch.backends.mps.is_available():
            return torch.device("mps")
        return torch.device("cpu")

    def _preprocess_frame(self, frame: Frame) -> torch.Tensor:
        """Preprocess a frame for the embedding model.

        Args:
            frame: Frame to preprocess.

        Returns:
            Preprocessed tensor ready for the model.
        """
        # OpenCV uses BGR, convert to RGB
        rgb_data = frame.data[:, :, ::-1]

        # Apply resize factor based on quality
        resize_factor = self.quality.image_resize_factor
        if resize_factor < 1.0:
            h, w = rgb_data.shape[:2]
            new_h, new_w = int(h * resize_factor), int(w * resize_factor)
            import cv2
            rgb_data = cv2.resize(rgb_data, (new_w, new_h))

        # Convert to PIL Image and apply transforms
        image = Image.fromarray(rgb_data)
        return self._transform(image)

    def compute_embedding(self, frame: Frame) -> EmbeddedFrame:
        """Compute the semantic embedding for a single frame.

        Args:
            frame: Frame to embed.

        Returns:
            EmbeddedFrame with the computed embedding.
        """
        tensor = self._preprocess_frame(frame).unsqueeze(0).to(self.device)

        with torch.no_grad():
            embedding = self._model(tensor)

        # Flatten and normalize
        embedding = embedding.squeeze().cpu().numpy()
        embedding = embedding / np.linalg.norm(embedding)

        return EmbeddedFrame(frame=frame, embedding=embedding.astype(np.float32))

    def compute_embeddings_batch(
        self,
        frames: Sequence[Frame],
        progress_callback: callable | None = None,
    ) -> list[EmbeddedFrame]:
        """Compute embeddings for a batch of frames.

        Args:
            frames: Sequence of frames to embed.
            progress_callback: Optional callback(current, total) for progress.

        Returns:
            List of EmbeddedFrames with computed embeddings.
        """
        if not frames:
            return []

        batch_size = self.quality.embedding_batch_size
        results: list[EmbeddedFrame] = []

        for i in range(0, len(frames), batch_size):
            batch_frames = frames[i:i + batch_size]
            batch_tensors = torch.stack([
                self._preprocess_frame(f) for f in batch_frames
            ]).to(self.device)

            with torch.no_grad():
                embeddings = self._model(batch_tensors)

            # Process each embedding
            for j, frame in enumerate(batch_frames):
                embedding = embeddings[j].squeeze().cpu().numpy()
                embedding = embedding / np.linalg.norm(embedding)
                results.append(EmbeddedFrame(
                    frame=frame,
                    embedding=embedding.astype(np.float32),
                ))

            if progress_callback is not None:
                progress_callback(len(results), len(frames))

        return results


def cosine_similarity(a: NDArray[np.float32], b: NDArray[np.float32]) -> float:
    """Compute cosine similarity between two normalized embedding vectors.

    Args:
        a: First embedding vector (normalized).
        b: Second embedding vector (normalized).

    Returns:
        Cosine similarity in range [-1, 1].
    """
    return float(np.dot(a, b))
