# AI Utils Documentation

This directory contains the complete documentation for AI Utils, built with [mdBook](https://rust-lang.github.io/mdBook/).

## Quick Start

### Prerequisites

- Rust and Cargo installed
- mdBook installed: `cargo install mdbook`

### Building Documentation

```bash
# Build the documentation
mdbook build

# Serve documentation locally (with live reload)
mdbook serve

# Watch for changes and rebuild automatically
mdbook watch
```

### Viewing Documentation

After building, the documentation will be available in the `book/` directory. Open `book/index.html` in your browser.

## Documentation Structure

```
docs/
├── src/                    # Source markdown files
│   ├── README.md          # Main documentation page
│   ├── getting-started/   # Installation and setup guides
│   │   ├── configuration.md
│   │   ├── installation.md
│   │   └── quick-start.md
│   └── modules/           # Module-specific documentation
│       └── overview.md
├── styles/                # Custom CSS styles
├── scripts/               # JavaScript files (Mermaid, etc.)
└── images/                # Documentation images
```

## Features

- **Mermaid Diagrams**: Interactive diagrams for architecture and flow visualization
- **Syntax Highlighting**: Code examples with proper syntax highlighting
- **Responsive Design**: Works on desktop and mobile devices
- **Search**: Full-text search across all documentation
- **Dark Mode**: Automatic dark mode support
- **Copy Code**: One-click code copying from examples

## Customization

### Adding New Pages

1. Create a new markdown file in the appropriate directory under `src/`
2. Add the page to `src/SUMMARY.md` to include it in the navigation
3. Rebuild the documentation

### Styling

Custom styles are in `styles/custom.css`. The documentation uses:
- Modern, clean design
- Consistent color scheme
- Responsive layout
- Accessibility features

### JavaScript

Custom JavaScript functionality is in `scripts/custom.js`:
- Mermaid diagram rendering
- Code copy buttons
- Search functionality
- Smooth scrolling

## Deployment

### GitHub Pages

The documentation is automatically deployed to GitHub Pages via the workflow in `.github/workflows/docs.yml`.

### Manual Deployment

```bash
# Build the documentation
mdbook build

# Deploy to any static hosting service
# The built files are in the book/ directory
```

## Contributing

When contributing to the documentation:

1. Follow the existing style and structure
2. Use Mermaid diagrams for visual explanations
3. Include practical code examples
4. Test that the documentation builds correctly
5. Update the table of contents in `book.toml` if adding new pages

## Local Development

```bash
# Install mdBook if not already installed
cargo install mdbook

# Start development server with live reload
mdbook serve --open

# Build for production
mdbook build
```

## Troubleshooting

### Common Issues

1. **Mermaid diagrams not rendering**: Ensure `mermaid.min.js` is in `scripts/`
2. **Styles not loading**: Check that `custom.css` is in `styles/`
3. **Build errors**: Verify all markdown files are valid and linked correctly

### Getting Help

- Check the [mdBook documentation](https://rust-lang.github.io/mdBook/)
- Review existing documentation for examples
- Open an issue for documentation-specific problems 