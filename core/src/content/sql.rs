//! SQL: a complete path from the first SELECT to joins, subqueries,
//! table design, transactions and a final database project.

use super::*;
use crate::model::Course;

pub fn course() -> Course {
    Course {
        id: "sql",
        title: "SQL",
        language: "sql",
        description: "A complete SQL path: querying, filtering, sorting, aggregates, joins, subqueries, changing data, designing tables, indexes, transactions and a final shop database.",
        color: "#8B5CF6",
        icon: "🗄️",
        sections: vec![
            first_queries(),
            filtering(),
            sorting_and_functions(),
            aggregates(),
            joins(),
            subqueries(),
            modifying(),
            design(),
            performance(),
            project(),
        ],
    }
}

fn first_queries() -> Section {
    section("sql-s1", "First Queries", vec![
        lesson("sql-1-1", "Databases and tables", vec![
            learn("Data in tables",
                "A relational database stores data in tables. Each table has columns (fields like name or price) and rows (one record each). SQL (Structured Query Language) is how you ask questions of it.",
                Some("-- table: products\n-- id | name    | price | category\n--  1 | Pen     |  1.50 | office\n--  2 | Lamp    | 24.00 | home\n--  3 | Stapler |  7.90 | office")),
            mc("In a table, what is one row?", None,
                &["A data type", "One record (e.g. one product)", "A column name", "A whole database"], 1,
                "Rows are records; columns are their fields."),
            mc("How many columns does the products table above have?", None, &["3", "4", "5", "1"], 1,
                "id, name, price and category."),
            mc("What does -- start in SQL?", None, &["A subtraction", "A comment", "A string", "A new table"], 1,
                "-- begins a comment that runs to the end of the line."),
        ]),
        lesson("sql-1-2", "SELECT", vec![
            learn("Reading a table",
                "SELECT picks columns and FROM names the table. * means every column. End statements with a semicolon. Keywords aren't case-sensitive, but writing them in caps is the convention.",
                Some("SELECT name, price FROM products;\nSELECT * FROM products;")),
            fill("Get every column from products.", "___ * ___ products;",
                &["SELECT", "FROM", "GET", "TABLE"], &["SELECT", "FROM"],
                "SELECT columns FROM table."),
            typein("Write a query that selects the title column from books.", None,
                &["SELECT title FROM books;", "SELECT title FROM books", "select title from books;", "select title from books"],
                "SELECT title FROM books;"),
            order("Order the query to show names and prices.",
                &["SELECT name, price", "FROM products;"],
                "SELECT comes before FROM."),
        ]),
        lesson("sql-1-3", "Aliases and DISTINCT", vec![
            learn("Cleaner results",
                "AS gives a column (or table) a new name in the result. DISTINCT removes duplicate rows.",
                Some("SELECT name AS product, price AS cost FROM products;\nSELECT DISTINCT category FROM products;  -- office, home")),
            mc("What does SELECT DISTINCT category FROM products return for the table above?", None,
                &["office, home, office", "office, home", "3 rows", "category"], 1,
                "Duplicates are removed."),
            fill("Rename the column in the output.", "SELECT price ___ cost FROM products;",
                &["AS", "TO", "IS", "="], &["AS"],
                "AS creates an alias."),
        ]),
    ])
}

fn filtering() -> Section {
    section("sql-s2", "Filtering Rows", vec![
        lesson("sql-2-1", "WHERE", vec![
            learn("Only some rows",
                "WHERE keeps rows that match a condition. Use = (not ==), <>, <, >, <=, >=. Text values go in single quotes.",
                Some("SELECT * FROM products WHERE price < 10;\nSELECT name FROM users WHERE country = 'IL';\nSELECT * FROM orders WHERE status <> 'shipped';")),
            mc("Which query finds products under 20?", None,
                &["SELECT * FROM products IF price < 20;", "SELECT * FROM products WHERE price < 20;",
                  "SELECT price < 20 FROM products;", "FILTER products WHERE price < 20;"], 1,
                "WHERE comes after FROM."),
            order("Order the query to get names of adults.",
                &["SELECT name", "FROM people", "WHERE age >= 18;"],
                "SELECT, then FROM, then WHERE."),
            fill("Find users from France.", "SELECT * FROM users WHERE country ___ ___;",
                &["=", "'FR'", "==", "\"FR\""], &["=", "'FR'"],
                "SQL compares with a single = and text uses single quotes."),
        ]),
        lesson("sql-2-2", "AND, OR, NOT", vec![
            learn("Combining conditions",
                "AND requires both conditions, OR either, NOT negates. Use parentheses to make the order clear.",
                Some("SELECT * FROM products\nWHERE category = 'office' AND price < 5;\n\nSELECT * FROM users\nWHERE (country = 'IL' OR country = 'US') AND NOT banned;")),
            mc("Which rows match price > 5 AND price < 10?", None,
                &["Prices 5 and 10", "Prices between 5 and 10, excluding both", "Every price", "None"], 1,
                "Both conditions must be true."),
            fill("Get office items OR anything under 2.", "SELECT * FROM products\nWHERE category = 'office' ___ price < 2;",
                &["OR", "AND", "||", "NOT"], &["OR"],
                "Either condition is enough."),
        ]),
        lesson("sql-2-3", "IN and BETWEEN", vec![
            learn("Shorter conditions",
                "IN (...) matches any value in a list. BETWEEN a AND b matches a range and includes both ends.",
                Some("SELECT * FROM users WHERE country IN ('IL', 'US', 'DE');\nSELECT * FROM products WHERE price BETWEEN 5 AND 10;")),
            mc("Does price BETWEEN 5 AND 10 include a price of 10?", None, &["Yes", "No", "Only for integers", "It's an error"], 0,
                "BETWEEN includes both boundaries."),
            fill("Match any of three categories.", "WHERE category ___ ('home', 'garden', 'toys')",
                &["IN", "ANY", "OF", "="], &["IN"],
                "IN checks membership in a list."),
        ]),
        lesson("sql-2-4", "LIKE patterns", vec![
            learn("Searching text",
                "LIKE matches patterns: % means any number of characters, _ means exactly one.",
                Some("SELECT * FROM users WHERE email LIKE '%@gmail.com';\nSELECT * FROM products WHERE name LIKE 'S%';     -- starts with S\nSELECT * FROM codes WHERE code LIKE 'A_1';       -- A?1")),
            mc("Which pattern matches names starting with Jo?", None, &["'%Jo'", "'Jo%'", "'_Jo'", "'Jo_'"], 1,
                "% after Jo allows anything to follow."),
            mc("What does _ match in a LIKE pattern?", None,
                &["Any number of characters", "Exactly one character", "An underscore only", "A space"], 1,
                "_ is a single-character wildcard."),
            typein("Write the LIKE pattern (with quotes) for text containing cat anywhere.", None, &["'%cat%'", "\"%cat%\""],
                "% on both sides allows anything before and after."),
        ]),
        lesson("sql-2-5", "NULL", vec![
            learn("Missing values",
                "NULL means \"no value\". It isn't equal to anything, not even NULL, so = NULL never matches. Use IS NULL and IS NOT NULL.",
                Some("SELECT * FROM users WHERE phone IS NULL;\nSELECT * FROM users WHERE phone IS NOT NULL;\n-- WHERE phone = NULL   → matches nothing!")),
            mc("Which finds users without an email?", None,
                &["WHERE email = NULL", "WHERE email IS NULL", "WHERE email == ''", "WHERE NOT email"], 1,
                "Comparisons with NULL use IS NULL."),
            fill("Find orders that have a delivery date.", "SELECT * FROM orders WHERE delivered_at ___ ___ NULL;",
                &["IS", "NOT", "!=", "<>"], &["IS", "NOT"],
                "IS NOT NULL keeps rows with a value."),
        ]),
    ])
}

fn sorting_and_functions() -> Section {
    section("sql-s3", "Sorting and Calculations", vec![
        lesson("sql-3-1", "ORDER BY", vec![
            learn("Sorting results",
                "ORDER BY sorts results: ASC (default) for smallest first, DESC for largest first. Sort by several columns separated by commas.",
                Some("SELECT name, price FROM products ORDER BY price DESC;\nSELECT * FROM users ORDER BY country, name;")),
            fill("Show the most expensive products first.", "SELECT * FROM products ORDER BY price ___;",
                &["DESC", "ASC", "DOWN", "REVERSE"], &["DESC"],
                "DESC sorts from high to low."),
            mc("What is the default sort direction?", None, &["DESC", "ASC", "Random", "Insertion order"], 1,
                "Without a keyword, ORDER BY is ascending."),
            order("Order the clauses.",
                &["SELECT name", "FROM products", "WHERE price > 5", "ORDER BY name;"],
                "SELECT, FROM, WHERE, ORDER BY."),
        ]),
        lesson("sql-3-2", "LIMIT and OFFSET", vec![
            learn("Top N and pagination",
                "LIMIT caps how many rows come back. OFFSET skips rows, which is useful for pages of results.",
                Some("SELECT name FROM players ORDER BY score DESC LIMIT 3;   -- top 3\nSELECT * FROM posts ORDER BY created_at DESC LIMIT 10 OFFSET 20;  -- page 3")),
            fill("Show the 5 most expensive products.", "SELECT * FROM products\nORDER BY price ___\n___ 5;",
                &["DESC", "LIMIT", "ASC", "TOP"], &["DESC", "LIMIT"],
                "DESC sorts high to low and LIMIT keeps the first 5."),
            mc("With 10 posts per page, what OFFSET shows page 2?", None, &["2", "10", "20", "1"], 1,
                "Skip the first 10 rows."),
        ]),
        lesson("sql-3-3", "Expressions and CASE", vec![
            learn("Computed columns",
                "You can calculate in SELECT and use functions like UPPER, LENGTH and ROUND. CASE WHEN … THEN … ELSE … END creates values from conditions.",
                Some("SELECT name, price * 1.17 AS with_tax FROM products;\nSELECT UPPER(name), LENGTH(name) FROM users;\nSELECT name,\n  CASE WHEN price < 10 THEN 'cheap' ELSE 'pricey' END AS label\nFROM products;")),
            mc("What does ROUND(7.456, 1) return?", None, &["7", "7.4", "7.5", "7.46"], 2,
                "Rounded to one decimal place."),
            fill("Label stock levels.", "SELECT name,\n  ___ WHEN qty = 0 THEN 'out' ELSE 'in stock' ___\nFROM products;",
                &["CASE", "END", "IF", "DONE"], &["CASE", "END"],
                "CASE … END wraps the conditions."),
            typein("What does LENGTH('hello') return?", None, &["5"], "Five characters."),
        ]),
    ])
}

fn aggregates() -> Section {
    section("sql-s4", "Aggregates and Grouping", vec![
        lesson("sql-4-1", "COUNT, SUM, AVG, MIN, MAX", vec![
            learn("Summarizing",
                "Aggregate functions turn many rows into one value: COUNT counts, SUM adds, AVG averages, MIN and MAX find extremes.",
                Some("SELECT COUNT(*) FROM orders;\nSELECT SUM(total), AVG(total) FROM orders;\nSELECT MIN(price), MAX(price) FROM products;")),
            mc("What does COUNT(*) return?", None,
                &["The number of rows", "The sum of a column", "The first row", "The number of columns"], 0,
                "COUNT(*) counts rows."),
            typein("Which aggregate function computes an average?", None, &["AVG", "AVG()", "avg"],
                "AVG() returns the mean."),
            fill("Find the cheapest price.", "SELECT ___(price) FROM products;",
                &["MIN", "MAX", "LOW", "FIRST"], &["MIN"],
                "MIN returns the smallest value."),
        ]),
        lesson("sql-4-2", "GROUP BY", vec![
            learn("One result per group",
                "GROUP BY splits rows into groups with the same value and computes aggregates per group. Every selected column must be grouped or aggregated.",
                Some("SELECT category, COUNT(*) AS items, AVG(price) AS avg_price\nFROM products\nGROUP BY category;")),
            mc("For the products table (office, home, office), how many rows does GROUP BY category return?", None,
                &["1", "2", "3", "4"], 1,
                "One row per distinct category."),
            fill("Count users per country.", "SELECT country, ___(*)\nFROM users\n___ BY country;",
                &["COUNT", "GROUP", "SUM", "ORDER"], &["COUNT", "GROUP"],
                "Aggregate with COUNT, grouped by country."),
            order("Order the clauses.",
                &["SELECT category, SUM(price)", "FROM products", "GROUP BY category;"],
                "GROUP BY comes after FROM (and WHERE)."),
        ]),
        lesson("sql-4-3", "HAVING", vec![
            learn("Filtering groups",
                "WHERE filters rows before grouping; HAVING filters groups after aggregation.",
                Some("SELECT customer_id, COUNT(*) AS orders\nFROM orders\nGROUP BY customer_id\nHAVING COUNT(*) >= 5;")),
            mc("Which clause filters on an aggregate like COUNT(*) > 3?", None, &["WHERE", "HAVING", "ORDER BY", "LIMIT"], 1,
                "Aggregates are filtered with HAVING."),
            order("Order the full query.",
                &["SELECT country, COUNT(*)", "FROM users", "WHERE active = 1", "GROUP BY country", "HAVING COUNT(*) > 10;"],
                "SELECT, FROM, WHERE, GROUP BY, HAVING."),
        ]),
    ])
}

fn joins() -> Section {
    section("sql-s5", "Joining Tables", vec![
        lesson("sql-5-1", "Keys and relationships", vec![
            learn("Why several tables?",
                "Instead of repeating customer details on every order, store customers once and reference them. A primary key (id) uniquely identifies a row; a foreign key (customer_id) points to a row in another table.",
                Some("-- customers: id | name\n--             1 | Ana\n--             2 | Ben\n-- orders:    id | customer_id | total\n--           10 |           1 | 30.00\n--           11 |           1 | 12.50\n--           12 |           2 |  8.00")),
            mc("In orders, what is customer_id?", None, &["A primary key", "A foreign key", "An index", "A view"], 1,
                "It references the id of a row in customers."),
            mc("How many orders does Ana have in the data above?", None, &["1", "2", "3", "0"], 1,
                "Orders 10 and 11 have customer_id 1."),
        ]),
        lesson("sql-5-2", "INNER JOIN", vec![
            learn("Matching rows",
                "INNER JOIN combines rows from two tables where the ON condition matches. Rows without a match are left out. Prefix columns with the table name (or an alias) when names clash.",
                Some("SELECT c.name, o.total\nFROM orders AS o\nJOIN customers AS c ON c.id = o.customer_id;")),
            fill("Join orders to customers.", "SELECT customers.name, orders.total\nFROM orders\n___ customers ___ customers.id = orders.customer_id;",
                &["JOIN", "ON", "WITH", "WHERE"], &["JOIN", "ON"],
                "JOIN table ON condition."),
            mc("A customer has no orders. Do they appear in an INNER JOIN of customers and orders?", None,
                &["Yes, with NULLs", "No", "Twice", "Only with DISTINCT"], 1,
                "INNER JOIN keeps only matching rows."),
            order("Order the join query.",
                &["SELECT p.name, c.title", "FROM products AS p", "JOIN categories AS c", "ON c.id = p.category_id;"],
                "SELECT, FROM, JOIN, ON."),
        ]),
        lesson("sql-5-3", "LEFT JOIN", vec![
            learn("Keep everything on the left",
                "LEFT JOIN keeps every row from the left table, filling NULL where there's no match. Great for finding things that have nothing related.",
                Some("SELECT c.name, o.id\nFROM customers AS c\nLEFT JOIN orders AS o ON o.customer_id = c.id\nWHERE o.id IS NULL;   -- customers with no orders")),
            mc("What does LEFT JOIN do with left rows that have no match?", None,
                &["Drops them", "Keeps them with NULLs for the right columns", "Duplicates them", "Raises an error"], 1,
                "All left rows are kept."),
            fill("Find products that were never ordered.", "SELECT p.name\nFROM products p\n___ JOIN order_items i ON i.product_id = p.id\nWHERE i.id ___ NULL;",
                &["LEFT", "IS", "INNER", "="], &["LEFT", "IS"],
                "LEFT JOIN, then keep rows where the right side is NULL."),
        ]),
        lesson("sql-5-4", "Multiple joins and self joins", vec![
            learn("Chaining joins",
                "You can join many tables in one query. A self join joins a table to itself with two aliases, for example employees and their managers.",
                Some("SELECT c.name, p.name AS product, i.qty\nFROM orders o\nJOIN customers c ON c.id = o.customer_id\nJOIN order_items i ON i.order_id = o.id\nJOIN products p ON p.id = i.product_id;\n\nSELECT e.name, m.name AS manager\nFROM employees e\nLEFT JOIN employees m ON m.id = e.manager_id;")),
            mc("Why does a self join need two aliases?", None,
                &["It's faster", "To tell the two copies of the table apart", "SQL requires aliases always", "To avoid NULLs"], 1,
                "e and m refer to different roles of the same table."),
            mc("How many JOIN clauses connect 4 tables in a chain?", None, &["2", "3", "4", "1"], 1,
                "Each join adds one table to the first."),
        ]),
    ])
}

fn subqueries() -> Section {
    section("sql-s6", "Subqueries and Set Operations", vec![
        lesson("sql-6-1", "Subqueries in WHERE", vec![
            learn("A query inside a query",
                "A subquery in parentheses computes a value or list used by the outer query.",
                Some("SELECT name, price FROM products\nWHERE price > (SELECT AVG(price) FROM products);\n\nSELECT name FROM customers\nWHERE id IN (SELECT customer_id FROM orders WHERE total > 100);")),
            mc("What does the first query return?", None,
                &["All products", "Products priced above the average", "The average price", "The cheapest product"], 1,
                "The subquery computes the average; the outer query filters on it."),
            fill("Find customers who placed any order.", "SELECT name FROM customers\nWHERE id ___ (SELECT customer_id FROM orders);",
                &["IN", "=", "LIKE", "IS"], &["IN"],
                "The subquery returns a list, so use IN."),
        ]),
        lesson("sql-6-2", "Derived tables and CTEs", vec![
            learn("Naming intermediate results",
                "A subquery in FROM acts as a temporary table. A CTE (WITH name AS (...)) does the same with a readable name declared up front.",
                Some("WITH customer_totals AS (\n  SELECT customer_id, SUM(total) AS spent\n  FROM orders\n  GROUP BY customer_id\n)\nSELECT c.name, t.spent\nFROM customer_totals t\nJOIN customers c ON c.id = t.customer_id\nORDER BY t.spent DESC;")),
            fill("Start a common table expression.", "___ recent ___ (\n  SELECT * FROM orders WHERE created_at > '2026-01-01'\n)\nSELECT COUNT(*) FROM recent;",
                &["WITH", "AS", "LET", "="], &["WITH", "AS"],
                "WITH name AS (query)."),
            mc("Why use a CTE?", None,
                &["It's the only way to join", "It makes complex queries readable by naming steps", "It stores data permanently", "It speeds up every query"], 1,
                "CTEs break a query into named, readable parts."),
        ]),
        lesson("sql-6-3", "UNION", vec![
            learn("Stacking results",
                "UNION combines the rows of two queries with the same number of columns and removes duplicates. UNION ALL keeps duplicates (and is faster).",
                Some("SELECT email FROM customers\nUNION\nSELECT email FROM newsletter_subscribers;")),
            mc("What's the difference between UNION and UNION ALL?", None,
                &["UNION ALL removes duplicates", "UNION removes duplicates; UNION ALL keeps them", "No difference", "UNION ALL joins tables"], 1,
                "UNION deduplicates."),
            mc("What must both queries in a UNION have?", None,
                &["The same table", "The same number of columns (compatible types)", "A WHERE clause", "An ORDER BY each"], 1,
                "Rows are stacked, so their shapes must match."),
        ]),
    ])
}

fn modifying() -> Section {
    section("sql-s7", "Changing Data", vec![
        lesson("sql-7-1", "INSERT", vec![
            learn("Adding rows",
                "INSERT INTO table (columns) VALUES (...) adds a row. You can insert several rows at once.",
                Some("INSERT INTO products (name, price, category)\nVALUES ('Notebook', 3.20, 'office');\n\nINSERT INTO tags (name) VALUES ('new'), ('sale');")),
            fill("Add a user.", "___ ___ users (name, email)\nVALUES ('Ana', 'ana@example.com');",
                &["INSERT", "INTO", "ADD", "TO"], &["INSERT", "INTO"],
                "INSERT INTO table."),
            order("Order the statement.",
                &["INSERT INTO books (title, year)", "VALUES ('Dune', 1965);"],
                "Columns first, then the matching values."),
        ]),
        lesson("sql-7-2", "UPDATE", vec![
            learn("Changing rows",
                "UPDATE table SET column = value WHERE condition changes existing rows. Without WHERE, every row is updated!",
                Some("UPDATE products SET price = price * 0.9 WHERE category = 'office';\nUPDATE users SET email = 'new@example.com' WHERE id = 7;")),
            fill("Mark order 5 as shipped.", "UPDATE orders ___ status = 'shipped' ___ id = 5;",
                &["SET", "WHERE", "TO", "IF"], &["SET", "WHERE"],
                "UPDATE … SET … WHERE …"),
            mc("What happens with UPDATE products SET price = 0; (no WHERE)?", None,
                &["Nothing", "Every product's price becomes 0", "Only the first row changes", "Syntax error"], 1,
                "Without WHERE the update applies to all rows."),
        ]),
        lesson("sql-7-3", "DELETE", vec![
            learn("Removing rows",
                "DELETE FROM table WHERE condition removes matching rows. Like UPDATE, forgetting WHERE affects every row.",
                Some("DELETE FROM sessions WHERE expires_at < '2026-01-01';")),
            fill("Delete user 9.", "___ ___ users WHERE id = 9;",
                &["DELETE", "FROM", "REMOVE", "DROP"], &["DELETE", "FROM"],
                "DELETE FROM table WHERE …"),
            mc("What's the difference between DELETE FROM t and DROP TABLE t?", None,
                &["None", "DELETE removes rows; DROP removes the whole table", "DROP removes rows only", "DELETE is faster"], 1,
                "DROP TABLE removes the table and its structure."),
        ]),
    ])
}

fn design() -> Section {
    section("sql-s8", "Designing Tables", vec![
        lesson("sql-8-1", "CREATE TABLE", vec![
            learn("Defining structure",
                "CREATE TABLE defines columns and their types: INTEGER, REAL/DECIMAL for numbers, TEXT/VARCHAR for text, DATE/TIMESTAMP for time, BOOLEAN for true/false.",
                Some("CREATE TABLE products (\n  id INTEGER PRIMARY KEY,\n  name TEXT NOT NULL,\n  price DECIMAL(8, 2),\n  created_at TIMESTAMP\n);")),
            fill("Create the table.", "___ ___ tags (\n  id INTEGER PRIMARY KEY,\n  name TEXT\n);",
                &["CREATE", "TABLE", "NEW", "MAKE"], &["CREATE", "TABLE"],
                "CREATE TABLE name (columns)."),
            mc("Which type fits a price like 19.99 best?", None, &["TEXT", "DECIMAL(8, 2)", "BOOLEAN", "DATE"], 1,
                "DECIMAL stores exact numbers with fixed decimals, ideal for money."),
        ]),
        lesson("sql-8-2", "Constraints", vec![
            learn("Rules that protect data",
                "PRIMARY KEY uniquely identifies rows. NOT NULL forbids missing values. UNIQUE forbids duplicates. DEFAULT supplies a value. CHECK enforces a condition.",
                Some("CREATE TABLE users (\n  id INTEGER PRIMARY KEY,\n  email TEXT NOT NULL UNIQUE,\n  age INTEGER CHECK (age >= 0),\n  active BOOLEAN DEFAULT TRUE\n);")),
            mc("Which constraint stops two users from having the same email?", None, &["NOT NULL", "UNIQUE", "DEFAULT", "CHECK"], 1,
                "UNIQUE rejects duplicate values."),
            fill("Require a name and forbid negative stock.", "name TEXT ___ ___,\nstock INTEGER CHECK (stock >= 0)",
                &["NOT", "NULL", "REQUIRED", "UNIQUE"], &["NOT", "NULL"],
                "NOT NULL makes a column mandatory."),
        ]),
        lesson("sql-8-3", "Foreign keys", vec![
            learn("Linking tables safely",
                "A FOREIGN KEY makes the database check that referenced rows exist. ON DELETE CASCADE removes child rows when the parent is deleted.",
                Some("CREATE TABLE orders (\n  id INTEGER PRIMARY KEY,\n  customer_id INTEGER NOT NULL,\n  total DECIMAL(10, 2),\n  FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE CASCADE\n);")),
            fill("Link the column to customers.", "FOREIGN KEY (customer_id) ___ customers(id)",
                &["REFERENCES", "POINTS", "LINKS", "JOIN"], &["REFERENCES"],
                "FOREIGN KEY (col) REFERENCES table(col)."),
            mc("What happens if you insert an order with a customer_id that doesn't exist?", None,
                &["It's inserted anyway", "The database rejects it", "A customer is created", "It becomes NULL"], 1,
                "The foreign key constraint fails."),
        ]),
        lesson("sql-8-4", "Changing structure and normalization", vec![
            learn("Evolving and organizing",
                "ALTER TABLE adds or changes columns; DROP TABLE removes a table. Normalization means each fact is stored once: split repeated data into its own table and link with keys.",
                Some("ALTER TABLE users ADD COLUMN phone TEXT;\nDROP TABLE old_logs;\n\n-- Not normalized: orders(customer_name, customer_email, …)\n-- Normalized: customers(id, name, email) + orders(customer_id, …)")),
            fill("Add a column.", "___ TABLE products ADD COLUMN sku TEXT;",
                &["ALTER", "CHANGE", "UPDATE", "MODIFY"], &["ALTER"],
                "ALTER TABLE changes structure."),
            mc("Why move customer details out of the orders table?", None,
                &["To make queries longer", "So each customer's data is stored once and can't get out of sync", "Tables can't have text", "For security only"], 1,
                "That's the goal of normalization."),
        ]),
    ])
}

fn performance() -> Section {
    section("sql-s9", "Indexes, Transactions and Views", vec![
        lesson("sql-9-1", "Indexes", vec![
            learn("Faster lookups",
                "An index is like a book's index: it lets the database find rows without scanning the whole table. Index columns you often filter, join or sort by. Indexes speed up reads but slow writes slightly.",
                Some("CREATE INDEX idx_orders_customer ON orders(customer_id);\nCREATE UNIQUE INDEX idx_users_email ON users(email);")),
            mc("Which column is a good index candidate?", None,
                &["A column you never query", "customer_id, used in many joins", "A notes column with long text", "Every column"], 1,
                "Index what you search and join on."),
            fill("Create an index on products.category.", "___ ___ idx_cat ON products(category);",
                &["CREATE", "INDEX", "ADD", "KEY"], &["CREATE", "INDEX"],
                "CREATE INDEX name ON table(column)."),
        ]),
        lesson("sql-9-2", "Transactions", vec![
            learn("All or nothing",
                "A transaction groups statements so they all succeed or none do. BEGIN starts it, COMMIT saves it, ROLLBACK undoes it. Classic example: moving money between accounts.",
                Some("BEGIN;\nUPDATE accounts SET balance = balance - 100 WHERE id = 1;\nUPDATE accounts SET balance = balance + 100 WHERE id = 2;\nCOMMIT;")),
            order("Order a safe transfer.",
                &["BEGIN;", "UPDATE accounts SET balance = balance - 50 WHERE id = 1;", "UPDATE accounts SET balance = balance + 50 WHERE id = 2;", "COMMIT;"],
                "Start, make both changes, commit."),
            mc("What does ROLLBACK do?", None,
                &["Saves changes", "Undoes the changes made in the current transaction", "Deletes the table", "Restarts the server"], 1,
                "ROLLBACK cancels the transaction."),
        ]),
        lesson("sql-9-3", "Views", vec![
            learn("Saved queries",
                "A view is a named query you can SELECT from like a table. It simplifies repeated complex queries.",
                Some("CREATE VIEW big_spenders AS\nSELECT customer_id, SUM(total) AS spent\nFROM orders\nGROUP BY customer_id\nHAVING SUM(total) > 1000;\n\nSELECT * FROM big_spenders;")),
            fill("Create a view.", "CREATE ___ active_users ___\nSELECT * FROM users WHERE active = TRUE;",
                &["VIEW", "AS", "TABLE", "IS"], &["VIEW", "AS"],
                "CREATE VIEW name AS query."),
            mc("Does a regular view store its own copy of the data?", None,
                &["Yes", "No, it runs its query when used", "Only for joins", "Only the first time"], 1,
                "A view is a stored query, not stored rows."),
        ]),
        lesson("sql-9-4", "Window functions", vec![
            learn("Ranking without collapsing rows",
                "Window functions compute values across related rows while keeping every row. ROW_NUMBER() OVER (ORDER BY …) numbers rows; PARTITION BY restarts per group.",
                Some("SELECT name, category, price,\n  RANK() OVER (PARTITION BY category ORDER BY price DESC) AS rank_in_cat\nFROM products;")),
            mc("How is a window function different from GROUP BY?", None,
                &["It's slower", "It keeps each row while adding a computed column", "It deletes rows", "No difference"], 1,
                "GROUP BY collapses rows; window functions don't."),
            fill("Number rows from highest score.", "SELECT name, ROW_NUMBER() ___ (ORDER BY score DESC)\nFROM players;",
                &["OVER", "BY", "WITH", "ON"], &["OVER"],
                "Window functions use OVER (...)."),
        ]),
    ])
}

fn project() -> Section {
    section("sql-s10", "Project: Shop Database", vec![
        lesson("sql-10-1", "Designing the schema", vec![
            learn("Tables for a shop",
                "We need customers, products, orders and order_items (which products, and how many, are in each order).",
                Some("CREATE TABLE customers (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT UNIQUE);\nCREATE TABLE products  (id INTEGER PRIMARY KEY, name TEXT NOT NULL, price DECIMAL(8,2) CHECK (price >= 0));\nCREATE TABLE orders    (id INTEGER PRIMARY KEY, customer_id INTEGER REFERENCES customers(id), created_at TIMESTAMP);\nCREATE TABLE order_items (\n  order_id INTEGER REFERENCES orders(id),\n  product_id INTEGER REFERENCES products(id),\n  qty INTEGER NOT NULL CHECK (qty > 0),\n  PRIMARY KEY (order_id, product_id)\n);")),
            mc("Why does order_items exist?", None,
                &["To store customer emails", "An order can contain many products and a product can be in many orders", "To speed up inserts", "It's optional decoration"], 1,
                "It connects orders and products (a many-to-many relationship)."),
            mc("What does PRIMARY KEY (order_id, product_id) prevent?", None,
                &["Orders without products", "The same product appearing twice in one order", "Negative prices", "Deleting orders"], 1,
                "A composite primary key must be unique as a pair."),
        ]),
        lesson("sql-10-2", "Adding data", vec![
            learn("Seed the database",
                "Insert a few rows into each table, parents before children so foreign keys are satisfied.",
                Some("INSERT INTO customers (id, name, email) VALUES (1, 'Ana', 'ana@ex.com'), (2, 'Ben', 'ben@ex.com');\nINSERT INTO products (id, name, price) VALUES (1, 'Mug', 9.50), (2, 'Tea', 4.00);\nINSERT INTO orders (id, customer_id, created_at) VALUES (100, 1, '2026-10-01');\nINSERT INTO order_items VALUES (100, 1, 2), (100, 2, 3);")),
            mc("Why insert customers before orders?", None,
                &["Alphabetical order", "Orders reference customers, so they must exist first", "It's faster", "No reason"], 1,
                "The foreign key needs the customer row."),
            typein("What is the total of order 100 (2 mugs at 9.50 + 3 teas at 4.00)?", None, &["31", "31.0", "31.00"],
                "2 × 9.50 + 3 × 4.00 = 19 + 12 = 31."),
        ]),
        lesson("sql-10-3", "Order totals report", vec![
            learn("Joins plus aggregation",
                "Join order_items to products to get prices, multiply by quantity and sum per order.",
                Some("SELECT o.id, c.name, SUM(i.qty * p.price) AS total\nFROM orders o\nJOIN customers c ON c.id = o.customer_id\nJOIN order_items i ON i.order_id = o.id\nJOIN products p ON p.id = i.product_id\nGROUP BY o.id, c.name\nORDER BY total DESC;")),
            fill("Compute each line's cost and add them up.", "SELECT order_id, ___(i.qty ___ p.price) AS total",
                &["SUM", "*", "COUNT", "+"], &["SUM", "*"],
                "Quantity times price, summed."),
            order("Order the report query.",
                &["SELECT o.id, SUM(i.qty * p.price) AS total", "FROM orders o", "JOIN order_items i ON i.order_id = o.id", "JOIN products p ON p.id = i.product_id", "GROUP BY o.id;"],
                "SELECT, FROM, JOINs, GROUP BY."),
        ]),
        lesson("sql-10-4", "Best customers and best sellers", vec![
            learn("Business questions",
                "Combine everything: CTEs, joins, grouping and ordering. Try your own questions next: monthly revenue, products never sold, repeat customers. You can now write them yourself.",
                Some("WITH spend AS (\n  SELECT o.customer_id, SUM(i.qty * p.price) AS spent\n  FROM orders o\n  JOIN order_items i ON i.order_id = o.id\n  JOIN products p ON p.id = i.product_id\n  GROUP BY o.customer_id\n)\nSELECT c.name, s.spent\nFROM spend s JOIN customers c ON c.id = s.customer_id\nORDER BY s.spent DESC\nLIMIT 3;")),
            mc("Which query finds products never sold?", None,
                &["products INNER JOIN order_items", "products LEFT JOIN order_items … WHERE order_items.product_id IS NULL", "SELECT * FROM order_items", "DELETE FROM products"], 1,
                "LEFT JOIN keeps unsold products with NULLs on the right."),
            fill("Show only the top 3 customers.", "ORDER BY s.spent ___\n___ 3;",
                &["DESC", "LIMIT", "ASC", "TOP"], &["DESC", "LIMIT"],
                "Sort high to low and keep three rows."),
        ]),
    ])
}
