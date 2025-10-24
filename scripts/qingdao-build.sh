BUILD_DIR=qingdao-judge/build

cd "$(dirname "$0")/../qingdao-judge"

rm -rf build
cmake . && make && sudo make install
