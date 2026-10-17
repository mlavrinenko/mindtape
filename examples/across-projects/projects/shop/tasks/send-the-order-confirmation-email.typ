#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Send the order confirmation email",
  status: todo(2026, 9, 25),
  priority: 7,
  links: depends-on("fix-the-cart-total-rounding.typ"),
)

= Summary

The email quotes the cart total, so it waits on the rounding fix.
