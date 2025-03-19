use dom_content_extraction::{get_content, scraper::Html};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = r#"<!DOCTYPE html><html><body>
        <nav>Home | About</nav>
        <main>
            <article>
                <h1>Main Article</h1>
                <p>This is the primary content that contains enough text to maintain proper density metrics. The paragraph needs sufficient length to establish text-to-link ratio.</p>
                <p>Second paragraph adds more textual density to ensure the content extraction algorithm works correctly.</p>
                <a href="\#">Related link</a>
            </article>
        </main>
        <footer>Copyright 2024</footer>
    </body></html>"#;

    let document = Html::parse_document(html);
    let content = get_content(&document).unwrap();
    println!("{}", content);
    Ok(())
}
