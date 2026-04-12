# TASKS.md

---

## 🔴 Bugs & Fixes

*Urgent — work these before anything else*

- [] In editor mode, while moving something it highlights what it is on top instead of where the holded item would end
  up.
  Like if I am holding `X` and hovering over `A, B, C`, it highlights `B`. What I would want is to see `A, B, X, C` in
  that list and
  `X` being highlighted meanwhile the original place of `X` is still showed in gray tones (like how it is rn, don't
  change that)
- [] In editor mode you cant move categories because instead they get toggled.
- [] Error auto-scroll speed is too low and the length calculation seems wrong, it continues to scroll even after the
  message ended (it should stop).
- [] After article cache is deleted, remove the read list as well. Also for some reason it still shows feeds as fetched
  even after deletion, update the dynamic objects in the code as well so they are cleared as well.
- [] Sub categories and some new settings you have added are missing the `|` line at the left most which makes them look
  deattached.
- [] The connector right under sub categories are one space to the left compared to where they need to be.

---

## 🟡 Tech Debt

*Correctness, architecture, cleanup — do after bugs*

*(none)*

---

## 🟢 Features

*New functionality — lowest priority*

- [] If the title of a feed under `FeedList` is too long, auto scroll it while hovering over it and refresh to 0 if not.
  Same goes for article titles as well.
- [] Add an option to stop auto fetch at start, add an option to fetch all. Also show the last fetched time, like
  `1d ago`, `15m ago` etc.
- [] Add `X/Y` to the right side of the progress bar.
- [] Change the `updated X ago` text colors so it is a constant color and only the number is colored instead of the
  whole text.

---

## ✅ Completed

*Reset each time, only keep the recent tasks*