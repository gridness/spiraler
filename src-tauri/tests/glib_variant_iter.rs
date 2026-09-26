#![cfg(target_os = "linux")]

use glib::{variant::ToVariant, Variant};

fn strings() -> Variant {
    ["zero", "one", "two", "three", "four"].to_variant()
}

#[test]
fn string_iterator_next_and_next_back_read_borrowed_strings() {
    let variant = strings();
    let mut iter = variant.array_iter_str().unwrap();
    assert_eq!(iter.next(), Some("zero"));
    assert_eq!(iter.next_back(), Some("four"));
    assert_eq!(iter.next(), Some("one"));
    assert_eq!(iter.next_back(), Some("three"));
    assert_eq!(iter.next(), Some("two"));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[test]
fn string_iterator_nth_and_nth_back_skip_from_both_ends() {
    let variant = strings();
    let mut iter = variant.array_iter_str().unwrap();
    assert_eq!(iter.nth(1), Some("one"));
    assert_eq!(iter.nth_back(1), Some("three"));
    assert_eq!(iter.next(), Some("two"));
    assert_eq!(iter.nth(0), None);
    assert_eq!(iter.nth_back(0), None);
}

#[test]
fn string_iterator_last_reads_the_remaining_tail() {
    let variant = strings();
    assert_eq!(variant.array_iter_str().unwrap().last(), Some("four"));
    let mut iter = variant.array_iter_str().unwrap();
    assert_eq!(iter.next_back(), Some("four"));
    assert_eq!(iter.last(), Some("three"));
}

#[test]
fn empty_string_iterator_stays_exhausted() {
    let variant = Vec::<&str>::new().to_variant();
    let mut iter = variant.array_iter_str().unwrap();
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
    assert_eq!(iter.nth(1), None);
    assert_eq!(iter.nth_back(1), None);
    assert_eq!(iter.last(), None);
}
