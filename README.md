# Simple tool for comparing image pixels in Rust

## Motivation:
- This tool was originally a text comparison tool (similar to diff). 
- I realized it would be better to have a tool to compare images pixel by pixel to find minute differences in my image processing tools I build.

## Features:
- GUI:
    - **◀ Prev** and **▶ Next** buttons (or the left/right arrow keys) to jump between islands
    - A **yellow crosshair** marks the center of the currently selected island (cause pixels are small)
    - A **Zoom slider** to zoom in and out
    - **pan** around the image freely

- Image comparison:
    - Displays window of the first image with modified pixels to show the difference from the second image:
    - **Red-shifted pixels** indicate areas where the two images differ
    - Red Pixels are grouped into **islands** (1 or more red pixels that are adjacent).

## How to use
- `cargo run image1 image2`