fn main() {
    use git2::{Direction, Remote};
    let mut remote = Remote::create_detached("https://github.com/typhon-ci/typhon").unwrap();
    remote.connect(Direction::Fetch).unwrap();
    let default_branch = remote.default_branch().unwrap();
    let default_branch = std::str::from_utf8(&default_branch).unwrap();
    let list = remote.list().unwrap();
    let head = list
        .iter()
        .find(|head| head.name() == default_branch)
        .unwrap();
    let rev = head.oid();
    println!("{rev:?}");
}
