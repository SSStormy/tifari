#!/bin/bash

echo "===> Building server"
cd prod
cargo build --release
cd ..

echo "===> Building frontend"
cd frontend
npm run build
cd ..

echo "===> Packaging"
rm -rf release
mkdir release
cd release
cp ../prod/target/release/prod.exe tifari.exe
cp ../prod/target/release/prod tifari
cp -r ../frontend/build static
cd ..

echo "===> Done!"
