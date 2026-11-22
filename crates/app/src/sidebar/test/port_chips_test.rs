use crate::sidebar::port_chips::{MAX_PORTS_SHOWN, ports_shown};

// Block 4, part 0 (3): the ports must fit in the card: a few are named and
// the rest are counted.
#[test]
fn a_card_names_a_few_ports_and_counts_the_rest() {
    assert_eq!(ports_shown(&[]), (&[][..], 0));
    assert_eq!(ports_shown(&[3000]), (&[3000][..], 0));
    assert_eq!(
        ports_shown(&[3000, 5173, 8080]),
        (&[3000, 5173, 8080][..], 0)
    );
    let many = [3000, 5173, 8080, 8765, 9229];
    let (shown, more) = ports_shown(&many);
    assert_eq!(shown, &many[..MAX_PORTS_SHOWN]);
    assert_eq!(more, 2);
}
