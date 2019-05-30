#!/bin/bash
cd "$(dirname "$0")"/resources

google-chrome http://localhost:8000 & python3 -m http.server

cd -