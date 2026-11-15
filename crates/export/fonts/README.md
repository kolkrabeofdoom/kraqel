# Embedded PDF fonts

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
