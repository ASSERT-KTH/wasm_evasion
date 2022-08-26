import sys
import json

if __name__ == '__main__':

    features_string = "container-drop-i32.rand-x container-drop-i64.rand-x container-x-drop-i32.rand container-x-drop-i64.rand i32.use_of_global i64.use_of_global f32.use_of_global f64.use_of_global i32.unfold i64.unfold replace-with-i32-1 replace-with-i64-1 replace-with-f32-1 replace-with-f64-1 replace-with-v128-1 replace-with-i32.rand replace-with-i64.rand replace-with-f32.rand replace-with-f64.rand replace-with-ref-null-func replace-with-ref-null-extern remove-drop remove-nop remove-global.set.0 remove-global.set.1 remove-elem.drop.0 remove-elem.drop.1 remove-data.drop.0 remove-data.drop.1 i32.add-1 i64.add-1 i32.sub-1 i64.sub-1 i32.and-1 i64.and-1 i32.or-1 i64.or-1 i32.xor-1 i64.xor-1 i32.shl-1 i64.shl-1 i32.shr_u-1 i64.shr_u-1 i32.shr_s-1 i64.shr_s-1 container-x-nop i32.operator_or_neg1  i32.operator_or_neg_one1 i32.operator_or_commutative i64.operator_or_commutative i32.operator_and_commutative i64.operator_and_commutative select_same_branches i32.sub_zero i32.mul_1 i64.mul_1 f32.mul_1 f64.mul_1 i32.add_0 i64.add_0 f32.add_0 f64.add_0 i32.xor_0 i64.xor_0 i32.eq_0 i64.eq_0 i32.shl_by_0 i64.shl_by_0 i32.shr_u_by_0 i64.shr_u_by_0 i32.shr_s_by_0 i64.shr_s_by_0 i64.operator_or_neg1 i64.sub_zero i32.add-commutes i64.add-commutes f32.add-commutes f64.add-commutes i32.mul-commutes i64.mul-commutes f32.mul-commutes f64.mul-commutes i32.and-commutes i64.and-commutes i32.or-commutes i64.or-commutes i32.xor-commutes i64.xor-commutes i32.eq-commutes i64.eq-commutes i32.mul-associates i64.mul-associates i32.add-associates i32.and-associates i32.or-associates i64.or-associates i32.xor-associates i64.xor-associates i32.eq-associates i64.eq-associates i32.mul-by-2 i64.mul-by-2 i32.mul-by-4 i64.mul-by-4 i32.mul-by-8"
    features_string = features_string.split(" ")

    features = [
        ("remove_item_function", "wasm-mutate/remove_item_function")
    ] + [ (f, f"wasm-mutate/peep_hole,wasm-mutate/{f}") for f in features_string ]

    for f, feature in features:
        print(f"- {{   dbconn: \"{f}\", features: \"{feature}\"  }}")