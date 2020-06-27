
#[repr(C)]
struct JmpBuf {

}

struct Thread {
    id: i32,

    context: JmpBuf,
}
