# Native DOCX v1 fixtures

Synthetic corpus for KQ-060 / Issue #117, produced on 14 November 2026. No private manuscript. The
native Rust writer creates `generated.docx` directly from `roundtrip::tests::sample`: 41 inventory
nodes, 24 paragraph/title controls, 15 inventory properties. The regression test requires identical
package parts with a fresh native generation and semantic equality after real Office saves. ZIP
creator-OS metadata differs on Windows and Unix; repeated generation within one environment remains
byte-identical and is tested separately.

| File                       | Provenance                                                 | Expected change |
| -------------------------- | ---------------------------------------------------------- | --------------- |
| `generated.docx`           | Native `write_docx_roundtrip`                              | None            |
| `word-resaved.docx`        | Open generated file in Word 16.0.20527.20112, Save As DOCX | None            |
| `libreoffice-resaved.docx` | LibreOffice 26.8.3.1, Office Open XML Text conversion      | None            |
| `word-edited.docx`         | Edit `Original` to `x` in actual Word document, save       | That text only  |

Office results were retained with `scripts/docx-prototype/retain_fixture.py`, which neutralizes
creator, last modifier, core timestamps and ZIP timestamps. Document, styles, numbering, controls
and inventory XML bytes remain unchanged. There is no prototype-profile migration in these files.
The public Rust reader also verified each original Office result before retention. All four files
were rendered with `render_docx.py`; their single pages were visually inspected without clipping,
overlap or missing corpus text. Layout differences are outside the semantic contract.

To regenerate the initial file, explicitly set `KRAQEL_ROUNDTRIP_QA_FILE` to a scratch output path
and run `cargo test -p kraqel-export generate_native_roundtrip_fixture -- --ignored`. After actual
Office saving, use `verify_native_roundtrip_fixture -- --ignored` with the input path and, for the
edit, `KRAQEL_ROUNDTRIP_QA_EDIT=x`. These opt-in helpers are test-only; the application does not
automate Office or execute Python.

SHA-256:

```text
b74c5955237f9cc16c48b2a120845a1d62d407cd3a08629ab86f446253f4719b  generated.docx
72f736ca6902dde9c2a771114a1609c4758adc38fe0358a10dc88246a326df77  libreoffice-resaved.docx
3d75b2d11b7e45d2f43ba8b58b38dc7c32c45b7eaa6aee3b8bc62724817f3cf2  word-edited.docx
b655b7500542f860a8fa0b9934677361c72032188b852f0153d377f11f6345b9  word-resaved.docx
```
