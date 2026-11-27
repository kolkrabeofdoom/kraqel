# Embedded PDF fonts

## Screenplay A4

The screenplay renderer embeds four unmodified static JetBrains Mono TTFs from
[release 2.304, revision `cd5227bd1f61dff3bbd6c814ceaf7ffd95e947d9`](https://github.com/JetBrains/JetBrainsMono/tree/cd5227bd1f61dff3bbd6c814ceaf7ffd95e947d9/fonts/ttf).
Copyright 2020 The JetBrains Mono Project Authors. The complete upstream
[SIL Open Font License 1.1](jetbrains-mono/OFL.txt) accompanies the fonts. The renderer uses
embedded glyph advances for wrapping and checks each styled run before writing the PDF. The four
cuts have installable embedding permission. The regression sample checks Latin, Greek and Cyrillic
glyphs in all four cuts.

| File                         | SHA-256                                                            |
| ---------------------------- | ------------------------------------------------------------------ |
| JetBrainsMono-Regular.ttf    | `a0bf60ef0f83c5ed4d7a75d45838548b1f6873372dfac88f71804491898d138f` |
| JetBrainsMono-Bold.ttf       | `5590990c82e097397517f275f430af4546e1c45cff408bde4255dad142479dcb` |
| JetBrainsMono-Italic.ttf     | `9d0a1f7a708e6af183f1193b7e81d40da294f5c67682c085d8401c60aac8ded4` |
| JetBrainsMono-BoldItalic.ttf | `4039d5ce0ed225bf9c8b2c8c6436290ae2f356b7e90d70fa666227238324aa3b` |
| OFL.txt                      | `30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad` |

## Prose

KQ-065 uses unchanged static Noto Sans Regular, Bold, Italic and BoldItalic TTF files. Source:
https://github.com/notofonts/noto-fonts/tree/ffebf8c1ee449e544955a7e813c54f9b73848eac/hinted/ttf/NotoSans
Repository commit: `ffebf8c1ee449e544955a7e813c54f9b73848eac` (archived upstream).

The complete SIL Open Font License 1.1 is included in `OFL.txt` from that same revision. Fonts are
bundled with the application and fully embedded into PDFs without installation, network retrieval,
modification or font subsetting. The OS/2 embedding permission is checked in Rust tests. No claim of
complete Unicode or complex shaping support.

| File                    | Embedded version                                                                       | SHA-256                                                            |
| ----------------------- | -------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| NotoSans-Bold.ttf       | Version 2.008; ttfautohint (v1.8) -l 8 -r 50 -G 200 -x 14 -D latn -f none -a qsq -X "" | `c976e4b1b99edc88775377fcc21692ca4bfa46b6d6ca6522bfda505b28ff9d6a` |
| NotoSans-BoldItalic.ttf | Version 2.008; ttfautohint (v1.8) -l 8 -r 50 -G 200 -x 14 -D latn -f none -a qsq -X "" | `6edf4227ef0fa846aca70e86a307804ca4401741830f5b3af0f2554abe2b8466` |
| NotoSans-Italic.ttf     | Version 2.008; ttfautohint (v1.8) -l 8 -r 50 -G 200 -x 14 -D latn -f none -a qsq -X "" | `36cff144df01309dab648bea71baff9bb074026914afe63aeacc8bc90b67a28b` |
| NotoSans-Regular.ttf    | Version 2.008; ttfautohint (v1.8) -l 8 -r 50 -G 200 -x 14 -D latn -f none -a qsq -X "" | `b85c38ecea8a7cfb39c24e395a4007474fa5a4fc864f6ee33309eb4948d232d5` |

Embedded copyright notices:

Copyright 2015-2021 Google LLC. All Rights Reserved.

Copyright 2015-2022 Google LLC. All Rights Reserved.

The PDF font profile and acceptance limits are documented in
`docs/data-format/ILLUSTRATED_MANUSCRIPT.md`.
