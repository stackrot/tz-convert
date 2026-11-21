use std::io::{self, Write};

pub fn write<const N: usize>(output: &mut impl Write, rows: &[[String; N]]) -> io::Result<()> {
    let mut widths = [0; N];
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    let mut border = String::new();
    for width in widths {
        border.push('+');
        border.push_str(&"-".repeat(width + 2));
    }
    border.push('+');

    writeln!(output, "{border}")?;
    for row in rows {
        for (cell, width) in row.iter().zip(widths) {
            write!(output, "| {cell:<width$} ")?;
        }
        writeln!(output, "|")?;
        writeln!(output, "{border}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn pads_columns_and_separates_rows() {
        let rows = [["Timezone", "UTC offset"], ["Asia/Kathmandu", "+05:45:00"]]
            .map(|row| row.map(String::from));
        let mut output = Vec::new();
        super::write(&mut output, &rows).unwrap();
        assert_eq!(
            String::from_utf8(output).unwrap(),
            "+----------------+------------+\n\
             | Timezone       | UTC offset |\n\
             +----------------+------------+\n\
             | Asia/Kathmandu | +05:45:00  |\n\
             +----------------+------------+\n"
        );
    }
}
