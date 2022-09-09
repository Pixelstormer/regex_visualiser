use egui::{
    text::LayoutJob, CollapsingHeader, FontId, FontSelection, Grid, ScrollArea, TextFormat,
    TextStyle, Ui,
};

/// Displays a guide to regular expression syntax
pub fn syntax_guide(ui: &mut Ui) {
    CollapsingHeader::new("Syntax Guide")
        .show_background(true)
        .show(ui, |ui| {
            ui.heading("Syntax Guide");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Documentation of the supported regular expression syntax (");
                    ui.hyperlink_to(
                        "Source",
                        "https://docs.rs/regex/1.6.0/regex/index.html#syntax",
                    );
                    ui.label(")");
                });

                let monospace = FontSelection::from(TextStyle::Monospace).resolve(ui.style());
                matching_one_character(ui, monospace.clone());
                character_classes(ui, monospace.clone());
                composites(ui);
                repetitions(ui);
                empty_matches(ui, monospace.clone());
                grouping_and_flags(ui, monospace.clone());
                escape_sequences(ui);
                perl_character_classes(ui, monospace.clone());
                ascii_character_classes(ui, monospace);
            });
        });
}

fn matching_one_character(ui: &mut Ui, monospace: FontId) {
    CollapsingHeader::new("Matching One Character").show(ui, |ui| {
        Grid::new("matching_one_character")
            .num_columns(2)
            .show(ui, |ui| {
                ui.monospace(".");
                let mut job = LayoutJob::default();
                job.plaintext("Any character except new line (Includes new line with ");
                job.with_font("s", monospace.clone());
                job.plaintext(" flag)");
                ui.label(job);

                ui.end_row();

                ui.monospace(r"\d");
                let mut job = LayoutJob::default();
                job.plaintext("Digit (Equivalent to ");
                job.with_font(r"\p{Nd}", monospace.clone());
                job.plaintext(")");
                ui.label(job);

                ui.end_row();

                ui.monospace(r"\D");
                ui.label("Not digit");
                ui.end_row();

                ui.monospace(r"\pN");
                ui.label("One-letter name Unicode character class");
                ui.end_row();

                ui.monospace(r"\p{Greek}");
                ui.label("Unicode character class (General category or script)");
                ui.end_row();

                ui.monospace(r"\PN");
                ui.label("Negated one-letter name Unicode character class");
                ui.end_row();

                ui.monospace(r"\P{Greek}");
                ui.label("Negated Unicode character class (General category or script)");
                ui.end_row();
            });
    });
}

fn character_classes(ui: &mut Ui, monospace: FontId) {
    CollapsingHeader::new("Character Classes").show(ui, |ui| {
        Grid::new("character_classes")
            .num_columns(2)
            .show(ui, |ui| {
                ui.monospace("[xyz]");
                ui.label("A character class matching either x, y or z (Union)");
                ui.end_row();

                ui.monospace("[^xyz]");
                ui.label("A character class matching any character except x, y and z");
                ui.end_row();

                ui.monospace("[a-z]");
                ui.label("A character class matching any character in the range a-z");
                ui.end_row();

                ui.monospace("[[:alpha:]]");
                let mut job = LayoutJob::default();
                job.plaintext("ASCII character class (Equivalent to ");
                job.with_font("[A-Za-z]", monospace.clone());
                job.plaintext(")");
                ui.label(job);

                ui.end_row();

                ui.monospace("[[:^alpha:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Negated ASCII character class (Equivalent to ");
                job.with_font("[^A-Za-z]", monospace.clone());
                job.plaintext(")");
                ui.label(job);

                ui.end_row();

                ui.monospace("[x[^xyz]]");
                ui.label("Nested/grouping character class (Matching any character except y and z)");
                ui.end_row();

                ui.monospace("[a-x&&xyz]");
                ui.label("Intersection (Matching x or y)");
                ui.end_row();

                ui.monospace("[0-9&&[^4]]");
                ui.label("Subtraction using intersection and negative (Matching 0-9 except 4)");
                ui.end_row();

                ui.monospace("[0-9--4]");
                ui.label("Direct subtraction (Matching 0-9 except 4)");
                ui.end_row();

                ui.monospace("[a-g~~b-h]");
                ui.label("Symmetric difference (Matching a and h only)");
                ui.end_row();

                ui.monospace(r"[\[\]]");
                ui.label("Escaping in character classes (Matching [ or ])");
                ui.end_row();
            });

        let mut job = LayoutJob::default();
        job.plaintext("Any named character class may appear inside a bracketed ");
        job.with_font("[...]", monospace.clone());
        job.plaintext(" character class. For example, ");
        job.with_font(r"[\p{Greek}[:digit:]]", monospace.clone());
        job.plaintext(" matches any Greek or ASCII digit. ");
        job.with_font(r"[\p{Greek}&&\pL]", monospace.clone());
        job.plaintext(" matches Greek letters.");
        ui.label(job);

        ui.label("Precedence in character classes, from most binding to least:");
        let mut job = LayoutJob::default();
        job.plaintext("\t1. Ranges: ");
        job.with_font("a-cd", monospace.clone());
        job.plaintext(" == ");
        job.with_font("[a-c]d", monospace.clone());

        job.plaintext("\n\t2. Union: ");
        job.with_font("ab&&bc", monospace.clone());
        job.plaintext(" == ");
        job.with_font("[ab]&&[bc]", monospace.clone());

        job.plaintext("\n\t3. Intersection: ");
        job.with_font("^a-z&&b", monospace.clone());
        job.plaintext(" == ");
        job.with_font("^[a-z&&b]", monospace.clone());

        job.plaintext("\n\t4. Negation");
        ui.label(job);
    });
}

fn composites(ui: &mut Ui) {
    CollapsingHeader::new("Composites").show(ui, |ui| {
        Grid::new("composites").num_columns(2).show(ui, |ui| {
            ui.monospace("xy");
            ui.label("Concatenation (x followed by y)");
            ui.end_row();

            ui.monospace("x|y");
            ui.label("Alternation (x or y, prefer x)");
            ui.end_row();
        });
    });
}

fn repetitions(ui: &mut Ui) {
    CollapsingHeader::new("Repetitions").show(ui, |ui| {
        Grid::new("repetitions").num_columns(2).show(ui, |ui| {
            ui.monospace("x*");
            ui.label("Zero or more of x (Greedy)");
            ui.end_row();

            ui.monospace("x+");
            ui.label("One or more of x (Greedy)");
            ui.end_row();

            ui.monospace("x?");
            ui.label("Zero or one of x (Greedy)");
            ui.end_row();

            ui.monospace("x*?");
            ui.label("Zero or more of x (Ungreedy/lazy)");
            ui.end_row();

            ui.monospace("x+?");
            ui.label("One or more of x (Ungreedy/lazy)");
            ui.end_row();

            ui.monospace("x??");
            ui.label("Zero or one of x (Ungreedy/lazy)");
            ui.end_row();

            ui.monospace("x{n,m}");
            ui.label("At least n of x and at most m of x (Greedy)");
            ui.end_row();

            ui.monospace("x{n,}");
            ui.label("At least n of x (Greedy)");
            ui.end_row();

            ui.monospace("x{n}");
            ui.label("Exactly n of x");
            ui.end_row();

            ui.monospace("x{n,m}?");
            ui.label("At least n of x and at most m of x (Ungreedy/lazy)");
            ui.end_row();

            ui.monospace("x{n,}?");
            ui.label("At least n of x (Ungreedy/lazy)");
            ui.end_row();

            ui.monospace("x{n}?");
            ui.label("Exactly n of x");
            ui.end_row();
        });
    });
}

fn empty_matches(ui: &mut Ui, monospace: FontId) {
    CollapsingHeader::new("Empty Matches").show(ui, |ui| {
        Grid::new("empty_matches").num_columns(2).show(ui, |ui| {
            ui.monospace("^");
            ui.label(
                "The beginning of the text (Or the start of a line with multi-line mode enabled)",
            );
            ui.end_row();

            ui.monospace("$");
            ui.label("The end of the text (Or the end of a line with multi-line mode enabled)");
            ui.end_row();

            ui.monospace(r"\A");
            ui.label("Only the beginning of the text (Even with multi-line mode enabled)");
            ui.end_row();

            ui.monospace(r"\z");
            ui.label("Only the end of the text (Even with multi-line mode enabled)");
            ui.end_row();

            ui.monospace(r"\b");
            let mut job = LayoutJob::default();
            job.plaintext("A Unicode word boundary (");
            job.with_font(r"\w", monospace.clone());
            job.plaintext(" on one side and ");
            job.with_font(r"\W", monospace.clone());
            job.plaintext(", ");
            job.with_font(r"\A", monospace.clone());
            job.plaintext(" or ");
            job.with_font(r"\a", monospace);
            job.plaintext(" on the other)");
            ui.label(job);
            ui.end_row();

            ui.monospace(r"\B");
            ui.label("Not a Unicode word boundary");
            ui.end_row();
        });
    });
}

fn grouping_and_flags(ui: &mut Ui, monospace: FontId) {
    CollapsingHeader::new("Grouping And Flags").show(ui, |ui| {
        Grid::new("grouping_and_flags")
            .num_columns(2)
            .show(ui, |ui| {
                ui.monospace("(exp)");
                ui.label("Numbered capture group (Indexed by opening parenthesis)");
                ui.end_row();

                ui.monospace("(?P<name>exp)");
                let mut job = LayoutJob::default();
                job.plaintext("Named (Also numbered) capture group (Characters allowed for name: ");
                job.with_font(r"[_0-9a-zA-Z.\[\]]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("(?:exp)");
                ui.label("Non-capturing group");
                ui.end_row();

                ui.monospace("(?flags)");
                ui.label("Set flags within current group");
                ui.end_row();

                ui.monospace("(?flags:exp)");
                ui.label("Set flags for exp (Non-capturing)");
                ui.end_row();
            });

        let mut job = LayoutJob::default();
        job.plaintext("Flags are each a single character. For example, ");
        job.with_font("(?x)", monospace.clone());
        job.plaintext(" sets the flag ");
        job.with_font("x", monospace.clone());
        job.plaintext(" and ");
        job.with_font("(?-x)", monospace.clone());
        job.plaintext(" clears the flag ");
        job.with_font("x", monospace.clone());
        job.plaintext(". Multiple flags can be set or cleared at the same time: ");
        job.with_font("(?xy)", monospace.clone());
        job.plaintext(" sets both the ");
        job.with_font("x", monospace.clone());
        job.plaintext(" and ");
        job.with_font("y", monospace.clone());
        job.plaintext(" flags, and ");
        job.with_font("(?x-y)", monospace.clone());
        job.plaintext(" sets the ");
        job.with_font("x", monospace.clone());
        job.plaintext(" flag and clears the ");
        job.with_font("y", monospace.clone());
        job.plaintext(" flag.");
        ui.label(job);

        ui.label("All flags are disabled by default unless stated otherwise. They are:");

        Grid::new("flags").num_columns(2).show(ui, |ui| {
            ui.monospace("i");
            ui.label("Case-insensitive: Letters match both upper and lower case");
            ui.end_row();

            ui.monospace("m");
            let mut job = LayoutJob::default();
            job.plaintext("Multi-line mode: ");
            job.with_font("^", monospace.clone());
            job.plaintext(" and ");
            job.with_font("$", monospace.clone());
            job.plaintext(" match the beginnings and ends of lines");
            ui.label(job);
            ui.end_row();

            ui.monospace("s");
            let mut job = LayoutJob::default();
            job.plaintext("Allow ");
            job.with_font(".", monospace.clone());
            job.plaintext(" to match ");
            job.with_font(r"\n", monospace.clone());
            ui.label(job);
            ui.end_row();

            ui.monospace("U");
            let mut job = LayoutJob::default();
            job.plaintext("Swap the meaning of ");
            job.with_font("x*", monospace.clone());
            job.plaintext(" and ");
            job.with_font("x*?", monospace.clone());
            ui.label(job);
            ui.end_row();

            ui.monospace("u");
            ui.label("Unicode support (Enabled by default)");
            ui.end_row();

            ui.monospace("x");
            let mut job = LayoutJob::default();
            job.plaintext("Ignore whitespace and allow line comments (Comments start with ");
            job.with_font("#", monospace.clone());
            job.plaintext(")");
            ui.label(job);
            ui.end_row();
        });
    });
}

fn escape_sequences(ui: &mut Ui) {
    CollapsingHeader::new("Escape Sequences").show(ui, |ui| {
        Grid::new("escape_sequences").num_columns(2).show(ui, |ui| {
            ui.monospace(r"\*");
            ui.label(r"Literal *, works for any punctuation character: \.+*?()|[]{}^$");
            ui.end_row();

            ui.monospace(r"\a");
            ui.label(r"Bell (\x07)");
            ui.end_row();

            ui.monospace(r"\f");
            ui.label(r"Form feed (\x0C)");
            ui.end_row();

            ui.monospace(r"\t");
            ui.label("Horizontal tab");
            ui.end_row();

            ui.monospace(r"\n");
            ui.label("New line");
            ui.end_row();

            ui.monospace(r"\r");
            ui.label("Carriage return");
            ui.end_row();

            ui.monospace(r"\v");
            ui.label(r"Vertical tab (\x0B)");
            ui.end_row();

            ui.monospace(r"\123");
            ui.label("Octal character code (Up to three digits) (When enabled)");
            ui.end_row();

            ui.monospace(r"\x7F");
            ui.label("Hex character code (Exactly two digits)");
            ui.end_row();

            ui.monospace(r"\x{10FFFF}");
            ui.label("Any hex character code corresponding to a Unicode code point");
            ui.end_row();

            ui.monospace(r"\u007F");
            ui.label("Hex character code (Exactly four digits)");
            ui.end_row();

            ui.monospace(r"\u{7F}");
            ui.label("Any hex character code corresponding to a Unicode code point");
            ui.end_row();

            ui.monospace(r"\U0000007F");
            ui.label("Hex character code (Exactly eight digits)");
            ui.end_row();

            ui.monospace(r"\U{7F}");
            ui.label("Any hex character code corresponding to a Unicode code point");
            ui.end_row();
        });
    });
}

fn perl_character_classes(ui: &mut Ui, monospace: FontId) {
    CollapsingHeader::new("Perl Character Classes (Unicode Friendly)").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("These classes are based on the definitions provided in ");
            ui.hyperlink_to(
                "UTS#18",
                "https://www.unicode.org/reports/tr18/#Compatibility_Properties",
            );
            ui.label(":");
        });
        Grid::new("perl_character_classes")
            .num_columns(2)
            .show(ui, |ui| {
                ui.monospace(r"\d");
                let mut job = LayoutJob::default();
                job.plaintext("Digit (");
                job.with_font(r"\p{Nd}", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace(r"\D");
                ui.label("Not digit");
                ui.end_row();

                ui.monospace(r"\s");
                let mut job = LayoutJob::default();
                job.plaintext("Whitespace (");
                job.with_font(r"\p{White_Space}", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace(r"\S");
                ui.label("Not whitespace");
                ui.end_row();

                ui.monospace(r"\w");
                let mut job = LayoutJob::default();
                job.plaintext("Word character (");
                job.with_font(r"\p{Alphabetic}", monospace.clone());
                job.plaintext(" + ");
                job.with_font(r"\p{M}", monospace.clone());
                job.plaintext(" + ");
                job.with_font(r"\d", monospace.clone());
                job.plaintext(" + ");
                job.with_font(r"\p{Pc}", monospace.clone());
                job.plaintext(" + ");
                job.with_font(r"\p{Join_Control}", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace(r"\W");
                ui.label("Not word character");
                ui.end_row();
            });
    });
}

fn ascii_character_classes(ui: &mut Ui, monospace: FontId) {
    CollapsingHeader::new("ASCII Character Classes").show(ui, |ui| {
        Grid::new("ascii_character_classes")
            .num_columns(2)
            .show(ui, |ui| {
                ui.monospace("[[:alnum:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Alphanumeric (Equivalent to ");
                job.with_font("[0-9A-Za-z]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:alpha:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Alphabetic (Equivalent to ");
                job.with_font("[A-Za-z]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:ascii:]]");
                let mut job = LayoutJob::default();
                job.plaintext("ASCII (Equivalent to ");
                job.with_font(r"[\x00-\x7F]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:blank:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Blank (Equivalent to ");
                job.with_font(r"[\t ]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:cntrl:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Control (Equivalent to ");
                job.with_font(r"[\x00-\x1F\x7F]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:digit:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Digits (Equivalent to ");
                job.with_font("[0-9]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:graph:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Graphical (Equivalent to ");
                job.with_font("[!-~]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:lower:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Lower case (Equivalent to ");
                job.with_font("[a-z]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:print:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Printable (Equivalent to ");
                job.with_font("[ -~]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:punct:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Punctuation (Equivalent to ");
                job.with_font(r"[!-/:-@\[-`{-~]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:space:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Whitespace (Equivalent to ");
                job.with_font(r"[\t\n\v\f\r ]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:upper:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Upper case (Equivalent to ");
                job.with_font("[A-Z]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:word:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Word characters (Equivalent to ");
                job.with_font("[0-9A-Za-z_]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();

                ui.monospace("[[:xdigit:]]");
                let mut job = LayoutJob::default();
                job.plaintext("Hex digit (Equivalent to ");
                job.with_font("[0-9A-Fa-f]", monospace.clone());
                job.plaintext(")");
                ui.label(job);
                ui.end_row();
            });
    });
}

trait LayoutJobShorthandsExt {
    fn plaintext(&mut self, text: &str);
    fn with_font(&mut self, text: &str, font: FontId);
}

impl LayoutJobShorthandsExt for LayoutJob {
    fn plaintext(&mut self, text: &str) {
        self.append(text, 0.0, Default::default());
    }

    fn with_font(&mut self, text: &str, font_id: FontId) {
        self.append(
            text,
            0.0,
            TextFormat {
                font_id,
                ..Default::default()
            },
        );
    }
}
