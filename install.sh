#!/bin/bash

# This script installs the required dependencies for the ray tracer and
# and then builds the extension with cython

pip install -r requirements.txt

python3 cython_extension/setup.py build_ext --inplace
