# StarDict ECDICT

This directory contains the read-only StarDict ECDICT resources used by Bugoo.

Current extracted size in this workspace is about 190 MB. Keep these files out of normal source
commits unless the project adopts Git LFS or a release-asset download pipeline.

Expected files:

```txt
stardict-ecdict-2.4.2.ifo
stardict-ecdict-2.4.2.idx
stardict-ecdict-2.4.2.dict
```

Current source package:

```txt
ecdict-stardict-28.zip
```

Source:

```txt
https://github.com/skywind3000/ECDICT/releases
```

The app reads the extracted StarDict files directly from this directory.

Do not commit vendor API keys or generated user data into this directory.
ECDICT is MIT licensed. Keep attribution in the app's About / Licenses section before public
distribution.
