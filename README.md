# Sunny Bites Self-Ordering Kiosk

A touch-screen self-ordering kiosk for quick-service restaurants. Customers walk up, browse the
menu, make their order exactly how they like it, and pay — without waiting in line.

> *Sunny Bites* is a fictional brand used for demonstration.

## What it does

- **Easy ordering for everyone** — big, friendly buttons and clear pictures, made for touch screens.
- **Always know where you are** — guided steps, a back button and a home button on every page.
- **Make it your way** — choose flavors, sides, drinks and add-ons for each item.
- **Find anything fast** — search the whole menu with the on-screen keyboard.
- **Helpful suggestions** — a gentle "make it a complete meal?" prompt, only when the customer wants it.
- **Dine in or take out** — customers pick once and can change their mind any time.
- **Flexible payment** — pay at the counter, by card or by e-wallet.
- **Order numbers** — every order gets a number to call when the food is ready.
- **Resets itself** — returns to the welcome screen when a customer walks away.

## Try it

1. Install [Rust](https://www.rust-lang.org/tools/install).
2. Download this project and open a terminal in its folder.
3. Run:

   ```sh
   cargo run --release
   ```

The kiosk opens in a window on your computer. On a real kiosk screen, start it in full-screen mode:

```sh
cargo run --release -- --kiosk
```

## Make it yours

The menu — store name, categories, items, prices, pictures and options — lives in a single file,
`assets/menu.json`. Edit it to match your restaurant; no programming needed.

## Good to know

This project is a working demo. Card and e-wallet payments are simulated, so connect a real
payment terminal before using it with customers.

## License

Released under the [Apache License 2.0](LICENSE).

It also includes the [Inter](https://rsms.me/inter/) font (SIL Open Font License) and food pictures
from [Noto Emoji](https://github.com/googlefonts/noto-emoji) (Apache License 2.0).
