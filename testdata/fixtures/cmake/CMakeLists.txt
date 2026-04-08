cmake_minimum_required(VERSION 3.28)
project(kat LANGUAGES CXX)

add_executable(kat main.cc)
target_compile_features(kat PRIVATE cxx_std_20)
