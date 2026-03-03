#!/bin/bash
#
for file in ./ui-assets/*.png; do
    ffmpeg -y -i "$file" -vf "format=yuv444p,format=rgb565le" \
        -f rawvideo "./src/ui/images/$(basename "${file%.png}").raw"
done
