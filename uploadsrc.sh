#!/bin/bash

timestamp=$(date +"%Y%m%d_%H%M%S")
git add .
git commit -m "$timestamp"
git push
