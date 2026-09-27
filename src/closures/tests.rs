use super::*;

fn fixture() -> Closures {
    Closures::of(vec![
        ("luna", vec![("libc", 100), ("bash", 50), ("nginx", 10)]),
        ("sol", vec![("libc", 100), ("bash", 50), ("postgres", 400)]),
    ])
}

fn total(paths: usize, size: u64) -> Total {
    Total { paths, size }
}

fn split(shared: u64, partial: u64, unique: u64) -> Split {
    Split {
        shared,
        partial,
        unique,
    }
}

fn share(name: &str, size: u64, holders: usize) -> Share {
    Share {
        name: name.into(),
        size,
        holders,
    }
}

#[test]
fn totals_are_derived_from_the_path_list() {
    assert_eq!(fixture().hosts["luna"].total(), total(3, 160));
}

#[test]
fn shared_is_what_every_host_carries() {
    assert_eq!(fixture().shared(), total(2, 150));
}

#[test]
fn unique_excludes_anything_another_host_also_has() {
    let c = fixture();
    assert_eq!(c.unique("luna"), total(1, 10));
    assert_eq!(c.unique("sol"), total(1, 400));
    assert_eq!(c.unique("nope"), Total::default());
}

#[test]
fn deduplication_counts_a_shared_path_once() {
    let c = fixture();
    assert_eq!(c.deduped(), total(4, 560));
    assert_eq!(c.naive_sum(), 710);
}

#[test]
fn largest_is_size_descending_and_bounded() {
    let c = fixture();
    let top = c.hosts["sol"].largest(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].path, "postgres");
    assert_eq!(top[1].path, "libc");
}

#[test]
fn a_split_partitions_the_host_total() {
    let c = fixture();
    let s = c.split("luna");
    assert_eq!(s, split(150, 0, 10));
    assert_eq!(
        s.shared + s.partial + s.unique,
        c.hosts["luna"].total().size
    );
    assert_eq!(c.split("nope"), Split::default());
}

#[test]
fn path_shares_carries_the_holder_count_per_path() {
    let c = fixture();
    assert_eq!(
        c.path_shares("luna"),
        vec![
            share("libc", 100, 2),
            share("bash", 50, 2),
            share("nginx", 10, 1)
        ]
    );
    assert_eq!(c.path_shares("nope"), vec![]);
}

#[test]
fn package_shares_fold_outputs_and_keep_the_holder_count() {
    let c = Closures::of(vec![
        (
            "luna",
            vec![
                ("/nix/store/a-glibc-2.42-67", 100),
                ("/nix/store/b-glibc-2.42-67-bin", 40),
                ("/nix/store/c-nginx-1.28", 10),
            ],
        ),
        ("sol", vec![("/nix/store/a-glibc-2.42-67", 100)]),
    ]);
    assert_eq!(
        c.package_shares("luna"),
        vec![
            share("glibc", 100, 2),
            share("glibc", 40, 1),
            share("nginx", 10, 1)
        ]
    );
}

#[test]
fn a_third_host_makes_the_partial_band_possible() {
    let c = Closures::of(vec![
        ("luna", vec![("libc", 100), ("bash", 50), ("nginx", 10)]),
        ("sol", vec![("libc", 100), ("bash", 50), ("postgres", 400)]),
        ("terra", vec![("libc", 100), ("bash", 50), ("nginx", 10)]),
    ]);
    assert_eq!(c.split("luna"), split(150, 10, 0));
    assert_eq!(c.split("sol"), split(150, 0, 400));
}

#[test]
fn a_single_host_shares_everything_with_itself() {
    let c = Closures::of(vec![("only", vec![("libc", 100)])]);
    assert_eq!(c.shared(), total(1, 100));
    assert_eq!(c.deduped(), total(1, 100));
    assert_eq!(c.split("only"), split(100, 0, 0));
}
