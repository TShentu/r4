FROM ubuntu:jammy as builder
RUN mkdir -p /opt/rss-termius-v2-rank && \
    mkdir -p /opt/xgboost && \
    mkdir -p /opt/rank_model/

RUN apt-get update && \
    apt-get install build-essential -y && \
    apt-get install cmake -y && \
    apt install git -y && \
    apt install curl -y && \
    apt install pkg-config -y && \
    apt-get install libssl-dev -y && \
    apt-get install libcpprest-dev -y && \
    cd /opt/xgboost && \
    git clone --recursive https://github.com/dmlc/xgboost . && \
    git checkout -b release_2.0.0 origin/release_2.0.0 && \
    git submodule init && \
    git submodule update && \
    mkdir build && \
    cd build && \
    cmake ..  -DBUILD_STATIC_LIB=on && \
    make -j$(nproc) && \
    make install && \
    cd /opt && \
    rm -r xgboost 


RUN cd /opt && \
    curl -LJO https://github.com/google/googletest/archive/refs/tags/v1.14.0.tar.gz  && \
    tar -xvzf googletest-1.14.0.tar.gz && \
    cd /opt/googletest-1.14.0 && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make -j$(nproc) && \
    make install && \
    cd /opt && \
    rm -r googletest-1.14.0 && \
    rm googletest-1.14.0.tar.gz


RUN cd /opt && \
    curl -LJO https://github.com/mongodb/mongo-c-driver/releases/download/1.24.0/mongo-c-driver-1.24.0.tar.gz && \
    tar -xvzf mongo-c-driver-1.24.0.tar.gz && \
    cd mongo-c-driver-1.24.0/build && \
    cmake .. && \
    make -j$(nproc) && \
    make install && \
    cd /opt && \
    rm mongo-c-driver-1.24.0.tar.gz && \
    rm -r mongo-c-driver-1.24.0

RUN cd /opt && \
    curl -LJO  https://github.com/mongodb/mongo-cxx-driver/releases/download/r3.8.1/mongo-cxx-driver-r3.8.1.tar.gz && \
    tar -xvzf  mongo-cxx-driver-r3.8.1.tar.gz && \
    cd mongo-cxx-driver-r3.8.1/build && \
    cmake -DCMAKE_INSTALL_PREFIX=/usr/local .. && \
    make -j$(nproc) && \
    make install && \
    cd /opt && \
    rm mongo-cxx-driver-r3.8.1.tar.gz && \
    rm -r mongo-cxx-driver-r3.8.1

RUN cd /opt && \
    curl -LO https://github.com/google/sentencepiece/archive/refs/tags/v0.1.99.tar.gz && \
    tar -xvzf v0.1.99.tar.gz && \
    cd sentencepiece-0.1.99 && \
    mkdir build && \
    cd build  && \
    cmake .. && \
    make -j$(nproc) && \
    make install && \
    cd /opt && \
    rm -r sentencepiece-0.1.99 && \
    rm v0.1.99.tar.gz

RUN cd /opt && \
    mkdir -p /usr/local/include/cppitertools && \
    curl -LO https://github.com/ryanhaining/cppitertools/archive/refs/tags/v2.1.tar.gz && \
    tar xzvf v2.1.tar.gz && \
    cd cppitertools-2.1/ && \
    cp *.hpp /usr/local/include/cppitertools && \
    cp -r internal /usr/local/include/cppitertools && \
    cd /opt && \
    rm  v2.1.tar.gz  && \
    rm -r cppitertools-2.1

ADD train-rank/src /opt/rss-terminus-v2-rank/src
ADD train-rank/test /opt/rss-terminus-v2-rank/test
COPY train-rank/CMakeLists.txt /opt/rss-terminus-v2-rank/CMakeLists.txt
RUN cd /opt/rss-terminus-v2-rank && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make && \
    cp bertv2train /usr/local/bin && \
    cd /opt && \
    rm -r rss-termius-v2-rank

FROM ubuntu:jammy
# FROM alpine


COPY --from=builder /usr/local/bin/bertv2train /usr/local/bin/bertv2train
COPY  --from=builder /usr/local/lib/libmongocxx.so._noabi /usr/local/lib/libmongocxx.so._noabi
COPY  --from=builder /usr/local/lib/libbsoncxx.so._noabi  /usr/local/lib/libbsoncxx.so._noabi
COPY  --from=builder  /usr/local/lib/libmongoc-1.0.so.0  /usr/local/lib/libmongoc-1.0.so.0
COPY   --from=builder  /usr/local/lib/libbson-1.0.so.0  /usr/local/lib/libbson-1.0.so.0
COPY --from=builder /usr/lib/x86_64-linux-gnu/libgomp.so.1 /usr/lib/x86_64-linux-gnu/libgomp.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libcpprest.so.2.10  /usr/lib/x86_64-linux-gnu/libcpprest.so.2.10
COPY --from=builder /usr/lib/x86_64-linux-gnu/libbrotlidec.so.1 /usr/lib/x86_64-linux-gnu/libbrotlidec.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libbrotlienc.so.1 /usr/lib/x86_64-linux-gnu/libbrotlienc.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libbrotlicommon.so.1  /usr/lib/x86_64-linux-gnu/libbrotlicommon.so.1
COPY --from=builder /usr/lib/x86_64-linux-gnu/libicuuc.so.70  /usr/lib/x86_64-linux-gnu/libicuuc.so.70 
COPY --from=builder /usr/lib/x86_64-linux-gnu/libicudata.so.70 /usr/lib/x86_64-linux-gnu/libicudata.so.70
RUN mkdir /opt/rank_model/
CMD ["/usr/local/bin/bertv2train"]
# ENTRYPOINT ["tail", "-f", "/dev/null"]