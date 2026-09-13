//! Differential-testing aid: mirrors the Go filepath reference program.
use capy_core::gopath;

fn main() {
    let cases = ["", "a/b", "a//b", "a/./b", "a/b/../c", "a/../../b", "/a/../../b",
        "/..", "./a", "a/b/", "/", "..", "../..", "a/b/c/../../d", "/a//b//c/./../d/",
        "./././", "x/..", "/x/..", "../a/b", "a/../b/../c"];
    for c in cases {
        println!("C|{}|{}", c, gopath::clean(c));
    }
    let joins: Vec<Vec<&str>> = vec![vec!["a","b"], vec!["a/","/b"], vec!["","b"],
        vec!["a","../b"], vec![], vec!["/x","y","../z"], vec!["a","","b"]];
    for j in &joins {
        println!("J|[{}]|{}", j.join(" "), gopath::join(j));
    }
    let dirs = ["/a/b/c.txt", "c.txt", "/a/b/", "/", "a/", ""];
    for d in dirs {
        println!("D|{}|{}|{}|{}", d, gopath::dir(d), gopath::base(d), gopath::ext(d));
    }
}
