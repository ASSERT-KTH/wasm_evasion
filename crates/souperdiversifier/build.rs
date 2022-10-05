fn main() {
    //println!("cargo:rustc-link-lib=dylib=souperInst");
    //println!("cargo:rustc-link-lib=dylib=souperKVStore");
    //println!("cargo:rustc-link-lib=dylib=souperParser");
    //println!("cargo:rustc-link-lib=dylib=souperSMTLIB2");
    //println!("cargo:rustc-link-lib=dylib=souperInfer");
    //println!("cargo:rustc-link-lib=dylib=souperTool");
    println!("cargo:rustc-link-lib=dylib=bridge");   
    
    // TODO add , z3, alive, hiredis libs
    println!("cargo:rustc-link-search=native=libs/souper");
}