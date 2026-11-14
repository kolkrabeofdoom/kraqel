# Synthetic image export fixtures

Generated locally with Pillow for KQ-063; no personal images or real metadata.

- `animated.png`: two 3×2 RGBA APNG frames, red/blue, 100 ms, loop 0.
- `animated.webp`: equivalent two-frame lossless WebP.
- `metadata.png`: static 3×2 PNG with a deliberately synthetic `private` text chunk.
- `scene.png`: static 320×180 RGBA illustration rectangle for native UI/output checks.

The animation tests use actual animation containers, not a renamed extension. JPEG orientation and
MPO marker cases are generated directly by Rust tests. The export keeps no metadata from these
inputs; only decoded pixels and dimensions enter the derived PNG.
