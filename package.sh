#!/bin/bash

echo "===> Building server"
cd prod
cargo build --release
cd ..

echo "===> Building frontend"
cd frontend
rm -rf build
npm run build
cd ..

echo "===> Packaging"
rm -rf release
mkdir release
cd release
cp ../prod/target/release/prod.exe tifari.exe
cp ../prod/target/release/prod tifari
mv  ../frontend/build static
cd ..

echo "===> Done!"
