#!/usr/bin/env bash

export WORKDIR=$(pwd)
cd /tmp || exit
rm -rf /tmp/tipedia/
mv $WORKDIR/site_generator/example/data/site.toml /tmp/site.toml
rm -rf $WORKDIR/site_generator/example/
mkdir $WORKDIR/site_generator/example/
git clone https://github.com/longfangsong/tipedia

cp -r /tmp/tipedia/static $WORKDIR/site_generator/example/static
cp -r /tmp/tipedia/templates $WORKDIR/site_generator/example/templates

mkdir -p $WORKDIR/site_generator/example/data/zh/how
mkdir -p $WORKDIR/site_generator/example/data/zh/what
mkdir -p $WORKDIR/site_generator/example/data/zh/where
mkdir -p $WORKDIR/site_generator/example/data/zh/why
mkdir -p $WORKDIR/site_generator/example/data/zh/what_if

mv /tmp/tipedia/data/zh/how/AddInfoTable.md $WORKDIR/site_generator/example/data/zh/how/
mv "/tmp/tipedia/data/zh/how/启用 Golang 代码中的某个 FailPoint .md" $WORKDIR/site_generator/example/data/zh/how/

mv /tmp/tipedia/data/zh/what/AmendSchema.md $WORKDIR/site_generator/example/data/zh/what/
mv /tmp/tipedia/data/zh/what/CIStr.md $WORKDIR/site_generator/example/data/zh/what/
mv /tmp/tipedia/data/zh/what/Operator.md $WORKDIR/site_generator/example/data/zh/what/
mv /tmp/tipedia/data/zh/what/Operator-f5f4846e806f6bc8475f39c419dcc931.md $WORKDIR/site_generator/example/data/zh/what/

mv /tmp/tipedia/data/zh/where/Hash\ Join\ 时遇到内存不足时将数据\ spill\ 到磁盘.md $WORKDIR/site_generator/example/data/zh/where/
mv /tmp/tipedia/data/zh/where/TiDB\ 处理SQL.md $WORKDIR/site_generator/example/data/zh/where/

mv /tmp/tipedia/data/zh/why/pd-on-etcd.md $WORKDIR/site_generator/example/data/zh/why/

mv /tmp/tipedia/data/zh/what_if/all-pd-went-down.md $WORKDIR/site_generator/example/data/zh/what_if/
mv /tmp/tipedia/data/zh/about.md $WORKDIR/site_generator/example/data/zh/
mv /tmp/tipedia/data/zh/translation.toml $WORKDIR/site_generator/example/data/zh/

mkdir -p $WORKDIR/site_generator/example/data/en/how
mkdir -p $WORKDIR/site_generator/example/data/en/what
mkdir -p $WORKDIR/site_generator/example/data/en/where
mkdir -p $WORKDIR/site_generator/example/data/en/why
mkdir -p $WORKDIR/site_generator/example/data/en/what_if

mv /tmp/tipedia/data/en/how/AddInfoTable.md $WORKDIR/site_generator/example/data/en/how/
mv "/tmp/tipedia/data/en/how/Enable A failpoint in Golang.md" $WORKDIR/site_generator/example/data/en/how/

mv /tmp/tipedia/data/en/what/AmendSchema.md $WORKDIR/site_generator/example/data/en/what/
mv /tmp/tipedia/data/en/what/CIStr.md $WORKDIR/site_generator/example/data/en/what/

mv /tmp/tipedia/data/en/why/pd-on-etcd.md $WORKDIR/site_generator/example/data/en/why/
mv /tmp/tipedia/data/en/about.md $WORKDIR/site_generator/example/data/en/
mv /tmp/tipedia/data/en/translation.toml $WORKDIR/site_generator/example/data/en/
mv /tmp/site.toml $WORKDIR/site_generator/example/data/site.toml