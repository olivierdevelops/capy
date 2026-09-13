---
title: Capy for everyone
---

# Capy for everyone

You don't need to be a programmer to use Capy. If you've ever
wanted to keep a list of recipes / events / plans / anything in a
format that turns into something polished — Capy is built for that.

## What you write looks like writing

Compare these two ways of making a recipe card.

**The old way** — open a recipe-website editor, click through 12
form fields, fight with rich-text formatting, get a result that
doesn't look quite right.

**The Capy way** — type this:

```
recipe "Lemon olive oil cake"
    serves 8
    time "45 minutes"

    ingredient "all-purpose flour"   "1 1/2 cups"
    ingredient "olive oil"           "3/4 cup"
    ingredient "sugar"               "1 cup"

    step "Preheat oven to 350F."
    step "Whisk flour and sugar."
    step "Add oil and lemon zest."
    step "Bake for 35-40 minutes."

    tip "Glaze with powdered sugar and lemon juice."
end
```

Then run one command and get a beautiful HTML recipe card you can
print, email, or paste into a blog.

## The full vocabulary, on one page

The recipe DSL above has **exactly six words**:

| Word          | Means                                   |
|---------------|-----------------------------------------|
| `recipe`      | Start a new recipe with a title.        |
| `serves`      | How many people it serves.              |
| `time`        | How long it takes.                      |
| `ingredient`  | One ingredient: name and quantity.      |
| `step`        | One step in the method.                 |
| `tip`        | A bonus suggestion.                     |
| `end`         | End of the recipe.                      |

That's the whole language for recipes. Anyone — a teenager learning
to cook, a grandparent typing up family favorites, a food blogger —
can use it within five minutes.

## Four ready-made vocabularies

The four animated demos on the [home page](index.md) each have a
similar tiny vocabulary tailored to a real-world task:

| Demo | What you describe | What you get |
|---|---|---|
| **Recipe card** | A recipe in 5–10 lines | A polished HTML recipe with two-column ingredient list and highlighted tip |
| **Event invitation** | A party invite in 8–10 lines | A printable pastel HTML invite with RSVP info |
| **Weekly meal plan** | Seven dinners + notes | A clean green-and-white HTML grid to tape to the fridge |
| **Reading log** | A child's books + page counts | A bright HTML certificate with a progress bar and star ratings |

Each one has a "library" — a small file that defines the vocabulary
and the visual design. **The library is reusable**: write 100
recipes against the same library, they all look beautiful, and
restyling them all is a 5-minute edit in one place.

## Where libraries come from

Three options, in order of effort:

1. **Use a ready-made one.** The repo has 50+ libraries for common
   tasks. Drop in your content, run `capy run`, done.
2. **Ask an AI to make one.** Describe what you want — "I want to
   write party invitations with a 1950s diner theme" — and Claude
   / ChatGPT / any modern AI will draft a Capy library in a couple
   of minutes. See [for AI agents](ai-agents.md).
3. **Write your own.** A typical library is 30–100 lines and looks
   like the recipe library above. Worth learning if you want full
   control. See [library authoring](library-authoring.md).

## What Capy is *not*

- Not a website builder. It produces files (HTML, Markdown, text)
  that you put wherever you want.
- Not a no-code platform that locks you in. The output is yours —
  plain files, no proprietary format, no monthly subscription.
- Not magic. If your library doesn't have a `step` keyword, you
  can't write `step ...` and expect it to work. The vocabulary is
  whatever the library author put in.

## A 5-minute setup

```sh
# 1. Install (one-time, no admin permissions needed)
cargo install --git https://github.com/olivierdevelops/capy capy-cli

# 2. Clone the samples
git clone https://github.com/olivierdevelops/capy
cd capy/samples/recipe-card

# 3. Edit script.capy in any text editor — replace with your recipe
# 4. Run
capy run lib.capy script.capy > my-recipe.html
open my-recipe.html
```

That's it. The same five steps work for every other sample —
substitute `event-invite/`, `weekly-meal-plan/`, `reading-log/`, or
any of the 50+ samples in the [`samples/`](https://github.com/olivierdevelops/capy/tree/main/samples) directory.

## When Capy is overkill

If you'll only ever make ONE recipe / ONE invite / ONE plan, just
open a Word doc. Capy pays off when you'll make *many* of something
that should look the same — fridge magnets for every week of the
year, party invites that all match the family aesthetic, a yearly
reading log for each child.

That repetition is where every "I should automate this" thought
comes from. Capy is the easiest way to actually do it.
