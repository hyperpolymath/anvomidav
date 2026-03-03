FROM gcr.io/oss-fuzz-base/base-builder-rust@8e8c483db84b4bee98b60c0593521ed34d9990e8 # 2024-11-14

RUN apt-get update && apt-get install -y make autoconf automake libtool
COPY . $SRC/anvomidav
WORKDIR $SRC/anvomidav
COPY .clusterfuzzlite/build.sh $SRC/
