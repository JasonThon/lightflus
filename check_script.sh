#! /bin/sh

function check_module {
    module = $1
    cargo check --manifest-path src/$module/Cargo.toml --all-features
}

function test_module_lib {
    module = $1
    cargo test --manifest-path --lib src/$module/Cargo.toml --all-features
}

all_check_modules = ( "proto" "common" "coordinator" "worker" "apiserver" )

for module in ${all_check_modules[@]}
do
    check_module $module
done;

all_test_modules = ( "common" "coordinator" "worker" )

for module in ${all_test_modules[@]}
do
    test_module_lib $module
done;