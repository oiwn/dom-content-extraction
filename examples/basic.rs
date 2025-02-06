/* use dom_content_extraction::DensityTree;
use scraper::Html;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html_content = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test Page</title></head>
        <body>
            <nav>Menu Item 1 | Menu Item 2</nav>
            <div class="sidebar">Side content</div>
            <article class="main-content">
                This is the main article content.
                It has multiple paragraphs and should be extracted.
                <p>This is another paragraph with important information.</p>
                <a href="\#">Some link</a>
            </article>
            <footer>Copyright 2024</footer>
        </body>
        </html>
    "#;

    let document = Html::parse_document(html_content);
    let mut dtree = DensityTree::from_document(&document)?;
    dtree.calculate_density_sum()?;
    let extracted_content = dtree.extract_content(&document)?;
    println!("Extracted content:\n{}", extracted_content);

    Ok(())
} */

use dom_content_extraction::{get_content, scraper::Html};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* let html = r#"
        <!DOCTYPE html><html><body>
            <nav>Navigation</nav>
            <article>
                <h1>Main Article</h1>
                <p>This is the primary content that should be extracted.</p>
                <p>A second paragraph with more content details, and
                information that elaborates fdfdsfsdfs fsdfsdfsdfsdfsdf
                fsdfsdfs fsdfs fdfs fsdfsdf</p>
            </article>
            <footer>Footer</footer>
        </body></html>
    "#;

    let html = r#"<!DOCTYPE html><html><body>
       <header>
           <nav>Home | About | Contact</nav>
       </header>
       <aside>
           <ul>
               <li>Sidebar link 1</li>
               <li>Sidebar link 2</li>
           </ul>
       </aside>
       <main>
           <article>
               <h1>Main Article Title</h1>
               <p>This is the primary content paragraph that should be extracted. It contains actual meaningful text that would be considered the main content of the page.</p>
               <p>A second paragraph with more content details and information that elaborates on the main topic.</p>
               <a href="\#">Related link</a>
           </article>
       </main>
       <footer>Copyright 2024</footer>
    </body></html>"#; */

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
