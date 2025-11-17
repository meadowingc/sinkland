Inspired by https://herman.bearblog.dev/messing-with-bots/

# Sinkland

A playground of different traps for AI bots and scrapers.

## Features

### Scraper Trap (`/trap/:slug`)
An infinite honeypot for bad actors who scrape content without permission. Each page:
- Has a randomly selected chapter title from "Dr. Jekyll and Mr. Hyde"
- Contains 4-5 random paragraphs from the book
- Links to 2-7 other randomly generated trap pages
- Creates an endless maze that wastes scrapers' time and bandwidth

The content is dynamically generated on each request, so no two pages are the same, and scrapers will never reach the end.

## Usage

```bash
cargo run
```

Then visit:
- `http://localhost:3000/hello/YourName` - Test endpoint
- `http://localhost:3000/trap/anything` - Enter the infinite scraper maze

## How it Works

On first build, the book "The Strange Case of Dr. Jekyll and Mr. Hyde" is downloaded from Project Gutenberg. The application then:
1. Parses all chapter titles and paragraphs from the book
2. On each `/trap/:slug` request, randomly selects content
3. Generates new unique URLs for links to other trap pages
4. Creates an infinite graph of fake content

Perfect for punishing unauthorized scrapers!