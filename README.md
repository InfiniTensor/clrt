# clrt

[![CI](https://github.com/InfiniTensor/clrt/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/InfiniTensor/clrt/actions)
[![license](https://img.shields.io/github/license/InfiniTensor/clrt)](https://mit-license.org/)
![GitHub repo size](https://img.shields.io/github/repo-size/InfiniTensor/clrt)
![GitHub code size in bytes](https://img.shields.io/github/languages/code-size/InfiniTensor/clrt)

[![GitHub Issues](https://img.shields.io/github/issues/InfiniTensor/clrt)](https://github.com/InfiniTensor/clrt/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/InfiniTensor/clrt)](https://github.com/InfiniTensor/clrt/pulls)
![GitHub contributors](https://img.shields.io/github/contributors/InfiniTensor/clrt)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/InfiniTensor/clrt)

OpenCL™ 运行时。

## 环境配置

必须配置环境变量 `OPENCL_HEADERS` 和 `OPENCL_LIB` 以使用项目。

`OPENCL_HEADERS` 指向 OpenCL™ 头文件位置，即克隆 [OpenCL-Headers](https://github.com/KhronosGroup/OpenCL-Headers) 项目的路径。

`OPENCL_LIB` 是 OpenCL™ 库的存放的路径，可能是类似 `*/lib`、`*/lib64` 或 `*/lib/x64` 的路径。
