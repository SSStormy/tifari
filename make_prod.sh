#!/bin/bash

cd frontend
npm run build
cd ..

cd prod/
rm -rf static
mv ../frontend/build .
mv build static
