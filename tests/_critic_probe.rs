//! TEMPORARY playtest probe round 3 — DELETE BEFORE FINISHING.

use vimforge::game::session::GameSession;

#[test]
fn probe_l30_checklist_registration() {
    let mut s = GameSession::new(120, 40);
    s.feed_keys("1");
    s.feed_keys(":level 30<CR>");
    if let Some(t) = s.tutorial.as_mut() {
        t.levels_completed = (1..=29).collect();
    }
    let dump = |s: &GameSession, label: &str| {
        let mut v: Vec<_> = s
            .tutorial
            .as_ref()
            .unwrap()
            .commands_used
            .iter()
            .cloned()
            .collect();
        v.sort();
        println!("{label}: {:?}", v);
    };
    s.feed_keys("w");
    s.feed_keys("fc");
    s.feed_keys(";");
    s.feed_keys("%");
    s.feed_keys("J");
    s.feed_keys(":s/pipe/belt/<CR>");
    s.feed_keys("<C-a>");
    s.feed_keys("gUU");
    s.feed_keys("/wall<CR>");
    s.feed_keys("*");
    s.feed_keys("ma");
    s.feed_keys("x");
    s.feed_keys(".");
    s.feed_keys("dd");
    s.feed_keys("`a");
    s.feed_keys("<C-o>");
    s.feed_keys("cc<Esc>");
    s.feed_keys("qbic<Esc>q");
    s.feed_keys("@b");
    s.feed_keys("yy");
    s.feed_keys("p");
    s.feed_keys("v<Esc>");
    s.feed_keys("<C-v><Esc>");
    dump(&s, "after full checklist");
    s.tick(2);
    println!(
        "L30 complete? level={:?} freeplay={} msg='{}'",
        s.current_level(),
        s.app.freeplay_unlocked,
        s.app.status_message
    );
}

#[test]
fn probe_zz() {
    let mut s = GameSession::new(120, 40);
    s.feed_keys("1");
    s.feed_keys("ZZ");
    println!("ZZ: quit={} pending='{}'", s.quit, s.app.pending_keys);
    s.feed_keys("ZQ");
    println!("ZQ: quit={} pending='{}'", s.quit, s.app.pending_keys);
}
