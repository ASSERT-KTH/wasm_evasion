INPUT=$1
shift 1

cargo build --release --features wasm-mutate/all
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf_all mutate -s 100 -e --attempts 1000  --input $INPUT --oracle $@ 2>all.logs.txt 1> all.stats.txt

C=0
FEATURES="wasm-mutate/i32.add-commutes,wasm-mutate/i64.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=1
FEATURES="wasm-mutate/i32.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=2
FEATURES="wasm-mutate/i64.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=3
FEATURES="wasm-mutate/f64.add_0,wasm-mutate/i32.shr_s_by_0,wasm-mutate/i32.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=4
FEATURES="wasm-mutate/f64.mul-commutes,wasm-mutate/i32.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=5
FEATURES="wasm-mutate/i32.mul-by-2,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=6
FEATURES="wasm-mutate/modify_custom_section_name,wasm-mutate/modify_custom_section,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=7
FEATURES="wasm-mutate/i64.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=8
FEATURES="wasm-mutate/i64.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=9
FEATURES="wasm-mutate/i32.add-commutes,wasm-mutate/i32.shl_by_0,wasm-mutate/i64.mul-by-2,wasm-mutate/i64.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=10
FEATURES="wasm-mutate/select-invert,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=11
FEATURES="wasm-mutate/i32.mul-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=12
FEATURES="wasm-mutate/add_function,wasm-mutate/i32.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=13
FEATURES="wasm-mutate/i32.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=14
FEATURES="wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=15
FEATURES="wasm-mutate/f32.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=16
FEATURES="wasm-mutate/i32.mul-by-8,wasm-mutate/i64.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=17
FEATURES="wasm-mutate/add_function,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=18
FEATURES="wasm-mutate/i32.shl_by_0,wasm-mutate/i32.use_of_global,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=19
FEATURES="wasm-mutate/i32.eq-commutes,wasm-mutate/i64.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=20
FEATURES="wasm-mutate/i32.shr_u_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=21
FEATURES="wasm-mutate/i32.and-commutes,wasm-mutate/i32.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=22
FEATURES="wasm-mutate/add_type,wasm-mutate/i64.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=23
FEATURES="wasm-mutate/i32.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=24
FEATURES="wasm-mutate/i32.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=25
FEATURES="wasm-mutate/container-nop-x,wasm-mutate/f32.add-commutes,wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=26
FEATURES="wasm-mutate/i64.mul-by-4,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=27
FEATURES="wasm-mutate/f64.mul_1,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=28
FEATURES="wasm-mutate/i32.shr_u_by_0,wasm-mutate/i32.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=29
FEATURES="wasm-mutate/add_function,wasm-mutate/i64.or-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=30
FEATURES="wasm-mutate/f32.use_of_global,wasm-mutate/f64.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=31
FEATURES="wasm-mutate/i64.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=32
FEATURES="wasm-mutate/i32.add-associates,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=33
FEATURES="wasm-mutate/i32.and-commutes,wasm-mutate/i32.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=34
FEATURES="wasm-mutate/i32.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=35
FEATURES="wasm-mutate/i64.or-commutes,wasm-mutate/i64.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=36
FEATURES="wasm-mutate/code_motion_ifs,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=37
FEATURES="wasm-mutate/code_motion_loops,wasm-mutate/i32.shr_s_by_0,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=38
FEATURES="wasm-mutate/i64.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=39
FEATURES="wasm-mutate/container-x-drop-i64.rand,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=40
FEATURES="wasm-mutate/i32.eq_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=41
FEATURES="wasm-mutate/i64.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=42
FEATURES="wasm-mutate/i32.add-commutes,wasm-mutate/i32.xor-commutes,wasm-mutate/i64.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=43
FEATURES="wasm-mutate/i64.mul-by-2,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=44
FEATURES="wasm-mutate/container-x-drop-i64.rand,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=45
FEATURES="wasm-mutate/i32.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=46
FEATURES="wasm-mutate/i32.mul-by-2,wasm-mutate/i64.mul-associates,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=47
FEATURES="wasm-mutate/container-nop-x,wasm-mutate/i32.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=48
FEATURES="wasm-mutate/i32.or-commutes,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=49
FEATURES="wasm-mutate/i32.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=50
FEATURES="wasm-mutate/i64.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=51
FEATURES="wasm-mutate/container-x-nop,wasm-mutate/i64.and-commutes,wasm-mutate/i64.shr_u_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=52
FEATURES="wasm-mutate/i64.mul-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=53
FEATURES="wasm-mutate/add_type,wasm-mutate/container-nop-x,wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=54
FEATURES="wasm-mutate/i32.or-associates,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=55
FEATURES="wasm-mutate/i32.add_0,wasm-mutate/i32.and-commutes,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=56
FEATURES="wasm-mutate/container-x-nop,wasm-mutate/f32.add_0,wasm-mutate/i64.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=57
FEATURES="wasm-mutate/i32.drop-x,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=58
FEATURES="wasm-mutate/i64.add_0,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=59
FEATURES="wasm-mutate/code_motion_loops,wasm-mutate/i64.eq_0,wasm-mutate/peep_hole,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=60
FEATURES="wasm-mutate/i64.mul_1,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=61
FEATURES="wasm-mutate/container-nop-x,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=62
FEATURES="wasm-mutate/modify_custom_section_data,wasm-mutate/container-x-nop,wasm-mutate/peep_hole,wasm-mutate/modify_custom_section,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=63
FEATURES="wasm-mutate/container-x-drop-i64.rand,wasm-mutate/select-invert,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=64
FEATURES="wasm-mutate/i32.mul-by-8,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=65
FEATURES="wasm-mutate/i32.eq_0,wasm-mutate/i32.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=66
FEATURES="wasm-mutate/f64.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=67
FEATURES="wasm-mutate/i32.xor-commutes,wasm-mutate/i64.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=68
FEATURES="wasm-mutate/f64.add_0,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=69
FEATURES="wasm-mutate/container-x-nop,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=70
FEATURES="wasm-mutate/f64.add-commutes,wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=71
FEATURES="wasm-mutate/i32.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=72
FEATURES="wasm-mutate/i32.add-associates,wasm-mutate/i64.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=73
FEATURES="wasm-mutate/i64.or-associates,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=74
FEATURES="wasm-mutate/container-x-drop-i64.rand,wasm-mutate/i64.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=75
FEATURES="wasm-mutate/i32.mul-by-2,wasm-mutate/i64.mul_1,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=76
FEATURES="wasm-mutate/container-x-drop-i64.rand,wasm-mutate/i64.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=77
FEATURES="wasm-mutate/add_type,wasm-mutate/f64.mul_1,wasm-mutate/i64.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=78
FEATURES="wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=79
FEATURES="wasm-mutate/i32.xor-commutes,wasm-mutate/i64.operator_or_commutative,wasm-mutate/i64.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=80
FEATURES="wasm-mutate/i32.eq-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=81
FEATURES="wasm-mutate/i32.operator_or_commutative,wasm-mutate/i64.add-commutes,wasm-mutate/i64.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=82
FEATURES="wasm-mutate/f64.use_of_global,wasm-mutate/i32.shr_s_by_0,wasm-mutate/i64.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=83
FEATURES="wasm-mutate/add_type,wasm-mutate/i32.add_0,wasm-mutate/i32.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=84
FEATURES="wasm-mutate/container-nop-x,wasm-mutate/container-x-drop-i32.rand,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=85
FEATURES="wasm-mutate/i32.and-commutes,wasm-mutate/i32.operator_and_commutative,wasm-mutate/i64.add-commutes,wasm-mutate/i64.shr_u_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=86
FEATURES="wasm-mutate/add_function,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=87
FEATURES="wasm-mutate/i64.mul_1,wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=88
FEATURES="wasm-mutate/i32.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=89
FEATURES="wasm-mutate/i64.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=90
FEATURES="wasm-mutate/i32.add_0,wasm-mutate/i32.mul-by-8,wasm-mutate/i32.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=91
FEATURES="wasm-mutate/code_motion_loops,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=92
FEATURES="wasm-mutate/i64.xor-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=93
FEATURES="wasm-mutate/f32.mul-commutes,wasm-mutate/i32.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=94
FEATURES="wasm-mutate/i32.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=95
FEATURES="wasm-mutate/i32.mul-by-4,wasm-mutate/i32.shr_s_by_0,wasm-mutate/i32.shr_u_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=96
FEATURES="wasm-mutate/container-x-drop-i64.rand,wasm-mutate/i32.add-commutes,wasm-mutate/i32.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=97
FEATURES="wasm-mutate/code_motion_loops,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=98
FEATURES="wasm-mutate/modify_custom_section_data,wasm-mutate/i32.operator_or_commutative,wasm-mutate/peep_hole,wasm-mutate/modify_custom_section,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=99
FEATURES="wasm-mutate/container-x-drop-i32.rand,wasm-mutate/container-x-drop-i64.rand,wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=100
FEATURES="wasm-mutate/f32.mul-commutes,wasm-mutate/i32.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=101
FEATURES="wasm-mutate/container-x-drop-i32.rand,wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=102
FEATURES="wasm-mutate/container-x-nop,wasm-mutate/i32.add-commutes,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=103
FEATURES="wasm-mutate/i32.mul-by-8,wasm-mutate/i32.xor-commutes,wasm-mutate/i64.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=104
FEATURES="wasm-mutate/add_type,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=105
FEATURES="wasm-mutate/i64.shr_u_by_0,wasm-mutate/i64.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=106
FEATURES="wasm-mutate/i32.or-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=107
FEATURES="wasm-mutate/i64.shl_by_0,wasm-mutate/i64.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=108
FEATURES="wasm-mutate/i32.add_0,wasm-mutate/i32.eq_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=109
FEATURES="wasm-mutate/f32.add_0,wasm-mutate/i32.add-associates,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=110
FEATURES="wasm-mutate/i32.sub_zero,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=111
FEATURES="wasm-mutate/add_function,wasm-mutate/i64.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=112
FEATURES="wasm-mutate/add_type,wasm-mutate/code_motion_loops,wasm-mutate/i32.operator_or_commutative,wasm-mutate/peep_hole,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=113
FEATURES="wasm-mutate/f32.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=114
FEATURES="wasm-mutate/f64.mul-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=115
FEATURES="wasm-mutate/i64.shr_s_by_0,wasm-mutate/i64.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=116
FEATURES="wasm-mutate/i32.eq-commutes,wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=117
FEATURES="wasm-mutate/i32.shr_s_by_0,wasm-mutate/i64.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=118
FEATURES="wasm-mutate/container-x-drop-i64.rand,wasm-mutate/i32.add-associates,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=119
FEATURES="wasm-mutate/i32.shl_by_0,wasm-mutate/i32.xor_0,wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=120
FEATURES="wasm-mutate/i64.add_0,wasm-mutate/i64.eq-commutes,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=121
FEATURES="wasm-mutate/i32.sub_zero,wasm-mutate/i64.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=122
FEATURES="wasm-mutate/container-x-drop-i32.rand,wasm-mutate/i64.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=123
FEATURES="wasm-mutate/i32.eq_0,wasm-mutate/i64.mul-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=124
FEATURES="wasm-mutate/i32.use_of_global,wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=125
FEATURES="wasm-mutate/i64.eq-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=126
FEATURES="wasm-mutate/i64.eq_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=127
FEATURES="wasm-mutate/modify_custom_section_name,wasm-mutate/modify_custom_section,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=128
FEATURES="wasm-mutate/f32.mul_1,wasm-mutate/i32.add-associates,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=129
FEATURES="wasm-mutate/i32.sub_zero,wasm-mutate/i64.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=130
FEATURES="wasm-mutate/modify_custom_section_name,wasm-mutate/container-nop-x,wasm-mutate/peep_hole,wasm-mutate/modify_custom_section,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=131
FEATURES="wasm-mutate/container-x-drop-i32.rand,wasm-mutate/i32.drop-x,wasm-mutate/i32.mul-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=132
FEATURES="wasm-mutate/i32.drop-x,wasm-mutate/i32.eq-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=133
FEATURES="wasm-mutate/f64.mul_1,wasm-mutate/i32.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=134
FEATURES="wasm-mutate/f64.add-commutes,wasm-mutate/i64.add-commutes,wasm-mutate/i64.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=135
FEATURES="wasm-mutate/i32.shr_u_by_0,wasm-mutate/i64.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=136
FEATURES="wasm-mutate/code_motion_loops,wasm-mutate/i32.sub_zero,wasm-mutate/i64.add-commutes,wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=137
FEATURES="wasm-mutate/container-drop-i64.rand-x,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=138
FEATURES="wasm-mutate/i64.or-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=139
FEATURES="wasm-mutate/i32.mul-by-4,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=140
FEATURES="wasm-mutate/container-x-drop-i32.rand,wasm-mutate/i32.add_0,wasm-mutate/i32.eq-commutes,wasm-mutate/i32.sub_zero,wasm-mutate/i32.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=141
FEATURES="wasm-mutate/i32.xor_0,wasm-mutate/i64.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=142
FEATURES="wasm-mutate/i32.operator_and_commutative,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=143
FEATURES="wasm-mutate/f32.mul-commutes,wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=144
FEATURES="wasm-mutate/i32.eq-commutes,wasm-mutate/i64.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=145
FEATURES="wasm-mutate/i32.and-commutes,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=146
FEATURES="wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=147
FEATURES="wasm-mutate/i64.add-commutes,wasm-mutate/i64.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=148
FEATURES="wasm-mutate/i32.add-associates,wasm-mutate/i32.or-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=149
FEATURES="wasm-mutate/i32.operator_or_commutative,wasm-mutate/i32.shr_u_by_0,wasm-mutate/i64.sub_zero,wasm-mutate/i64.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=150
FEATURES="wasm-mutate/container-x-drop-i32.rand,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=151
FEATURES="wasm-mutate/container-nop-x,wasm-mutate/i64.eq_0,wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=152
FEATURES="wasm-mutate/i32.eq_0,wasm-mutate/i32.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=153
FEATURES="wasm-mutate/i32.or-commutes,wasm-mutate/i64.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=154
FEATURES="wasm-mutate/i32.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=155
FEATURES="wasm-mutate/modify_custom_section_data,wasm-mutate/modify_custom_section,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=156
FEATURES="wasm-mutate/i32.add_0,wasm-mutate/i32.mul_1_t5.snapshot,wasm-mutate/i32.shr_s_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=157
FEATURES="wasm-mutate/i32.xor_0,wasm-mutate/i64.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=158
FEATURES="wasm-mutate/i64.add-commutes,wasm-mutate/i64.xor-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=159
FEATURES="wasm-mutate/i64.unfold,wasm-mutate/select-invert,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=160
FEATURES="wasm-mutate/i32.sub_zero,wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=161
FEATURES="wasm-mutate/modify_custom_section_name,wasm-mutate/i32.eq_0,wasm-mutate/i32.mul-by-8,wasm-mutate/peep_hole,wasm-mutate/modify_custom_section,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=162
FEATURES="wasm-mutate/i64.shr_u_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=163
FEATURES="wasm-mutate/container-x-nop,wasm-mutate/i32.shr_u_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=164
FEATURES="wasm-mutate/i32.add-commutes,wasm-mutate/i64.shr_u_by_0,wasm-mutate/select-invert,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=165
FEATURES="wasm-mutate/i32.eq-commutes,wasm-mutate/i64.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=166
FEATURES="wasm-mutate/i64.operator_and_commutative,wasm-mutate/remove_item_global,wasm-mutate/peep_hole,wasm-mutate/remove_item,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=167
FEATURES="wasm-mutate/i32.use_of_global,wasm-mutate/i32.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=168
FEATURES="wasm-mutate/i64.eq-commutes,wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=169

FEATURES="wasm-mutate/i64.eq_0,wasm-mutate/i64.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=170
FEATURES="wasm-mutate/i32.sub_zero,wasm-mutate/i64.or-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=171
FEATURES="wasm-mutate/i32.mul-by-4,wasm-mutate/i64.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=172
FEATURES="wasm-mutate/i32.shr_s_by_0,wasm-mutate/i32.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=173
FEATURES="wasm-mutate/container-x-nop,wasm-mutate/i64.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=174
FEATURES="wasm-mutate/i32.shr_s_by_0,wasm-mutate/i32.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=175
FEATURES="wasm-mutate/add_type,wasm-mutate/f32.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=176
FEATURES="wasm-mutate/i32.and-associates,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=177
FEATURES="wasm-mutate/modify_custom_section_name,wasm-mutate/container-x-nop,wasm-mutate/peep_hole,wasm-mutate/modify_custom_section,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=178
FEATURES="wasm-mutate/i32.eq-commutes,wasm-mutate/i64.or-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=179
FEATURES="wasm-mutate/add_type,wasm-mutate/code_motion_ifs,wasm-mutate/container-drop-i32.rand-x,wasm-mutate/f64.use_of_global,wasm-mutate/i32.mul_1_t5.snapshot,wasm-mutate/peep_hole,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=180
FEATURES="wasm-mutate/i32.sub_zero,wasm-mutate/i64.eq-commutes,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=181
FEATURES="wasm-mutate/add_type,wasm-mutate/f32.mul-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=182
FEATURES="wasm-mutate/i32.drop-x,wasm-mutate/i32.mul-by-8,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=183
FEATURES="wasm-mutate/i32.and-commutes,wasm-mutate/i32.operator_and_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=184
FEATURES="wasm-mutate/f64.mul_1,wasm-mutate/i32.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=185
FEATURES="wasm-mutate/add_type,wasm-mutate/i32.xor-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=186
FEATURES="wasm-mutate/i64.add_0,wasm-mutate/i64.sub_zero,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=187
FEATURES="wasm-mutate/container-x-nop,wasm-mutate/i32.add_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=188
FEATURES="wasm-mutate/f64.mul_1,wasm-mutate/i64.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=189
FEATURES="wasm-mutate/i32.mul-by-4,wasm-mutate/i64.shl_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=190
FEATURES="wasm-mutate/select-invert,wasm-mutate/remove_item_function,wasm-mutate/peep_hole,wasm-mutate/remove_item,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=191
FEATURES="wasm-mutate/i32.mul-by-4,wasm-mutate/i32.shr_u_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=192
FEATURES="wasm-mutate/i32.mul_1_t5.snapshot,wasm-mutate/i64.and-commutes,wasm-mutate/i64.unfold,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=193
FEATURES="wasm-mutate/container-nop-x,wasm-mutate/i32.use_of_global,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=194
FEATURES="wasm-mutate/i32.eq-commutes,wasm-mutate/i32.unfold,wasm-mutate/i64.xor_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=195
FEATURES="wasm-mutate/code_motion_ifs,wasm-mutate/i32.eq-commutes,wasm-mutate/peep_hole,wasm-mutate/code_motion,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=196
FEATURES="wasm-mutate/container-x-nop,wasm-mutate/i64.mul-by-8,wasm-mutate/i64.operator_or_commutative,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=197
FEATURES="wasm-mutate/container-x-drop-i64.rand,wasm-mutate/i32.mul-by-2,wasm-mutate/i64.and-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=198
FEATURES="wasm-mutate/i32.xor-commutes,wasm-mutate/i64.shr_u_by_0,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

C=199
FEATURES="wasm-mutate/i32.add_0,wasm-mutate/i64.add-commutes,wasm-mutate/peep_hole,"
cargo build --release --features $FEATURES
RUST_LOG=debug ./target/release/analyzer --dbconn datas/cf$C mutate -s 100 -e --attempts 5  --input $INPUT --oracle $@ 2>$C.logs.txt 1> $C.stats.txt

