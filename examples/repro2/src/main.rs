//! f

fn main() {
    iced::application(|| (), update, view)
        .subscription(|_| {
            iced::window::events().filter_map(|e| {
                dbg!(e);
                None
            })
        })
        .run()
        .unwrap();
}

fn update(_: &mut (), _: ()) {}
fn view(_: &()) -> iced::Element<'_, ()> {
    "Reproducing...".into()
}
