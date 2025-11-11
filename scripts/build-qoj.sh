BUILD_DIR=qingdao-judge/build

cd "$(dirname "$0")/../qingdao-judger"

rm -rf build
cmake . && make && sudo make install
