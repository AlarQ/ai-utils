// Custom JavaScript for AI Utils documentation

// Initialize Mermaid diagrams
document.addEventListener('DOMContentLoaded', function() {
    // Check if Mermaid is available
    if (typeof mermaid !== 'undefined') {
        // Configure Mermaid
        mermaid.initialize({
            startOnLoad: true,
            theme: 'default',
            flowchart: {
                useMaxWidth: true,
                htmlLabels: true,
                curve: 'basis'
            },
            sequence: {
                useMaxWidth: true,
                diagramMarginX: 50,
                diagramMarginY: 10,
                actorMargin: 50,
                width: 150,
                height: 65,
                boxMargin: 10,
                boxTextMargin: 5,
                noteMargin: 10,
                messageMargin: 35,
                mirrorActors: true,
                bottomMarginAdj: 1,
                useMaxWidth: true,
                rightAngles: false,
                showSequenceNumbers: false
            },
            gantt: {
                titleTopMargin: 25,
                barHeight: 20,
                barGap: 4,
                topPadding: 50,
                leftPadding: 75,
                gridLineStartPadding: 35,
                fontSize: 11,
                fontFamily: '"Open-Sans", "sans-serif"',
                numberSectionStyles: 4,
                axisFormat: '%Y-%m-%d'
            }
        });
    }

    // Add copy button to code blocks
    addCopyButtonsToCodeBlocks();
    
    // Add syntax highlighting
    addSyntaxHighlighting();
    
    // Add smooth scrolling
    addSmoothScrolling();
});

// Add copy buttons to code blocks
function addCopyButtonsToCodeBlocks() {
    const codeBlocks = document.querySelectorAll('pre code');
    
    codeBlocks.forEach((codeBlock, index) => {
        const pre = codeBlock.parentElement;
        if (pre && !pre.querySelector('.copy-button')) {
            const copyButton = document.createElement('button');
            copyButton.className = 'copy-button';
            copyButton.textContent = 'Copy';
            copyButton.style.cssText = `
                position: absolute;
                top: 8px;
                right: 8px;
                padding: 4px 8px;
                background: #007acc;
                color: white;
                border: none;
                border-radius: 4px;
                font-size: 12px;
                cursor: pointer;
                opacity: 0;
                transition: opacity 0.2s;
            `;
            
            pre.style.position = 'relative';
            pre.appendChild(copyButton);
            
            pre.addEventListener('mouseenter', () => {
                copyButton.style.opacity = '1';
            });
            
            pre.addEventListener('mouseleave', () => {
                copyButton.style.opacity = '0';
            });
            
            copyButton.addEventListener('click', async () => {
                try {
                    await navigator.clipboard.writeText(codeBlock.textContent);
                    copyButton.textContent = 'Copied!';
                    setTimeout(() => {
                        copyButton.textContent = 'Copy';
                    }, 2000);
                } catch (err) {
                    console.error('Failed to copy: ', err);
                }
            });
        }
    });
}

// Add syntax highlighting
function addSyntaxHighlighting() {
    // This would integrate with a syntax highlighter like Prism.js
    // For now, we'll just add some basic styling
    const codeBlocks = document.querySelectorAll('pre code');
    
    codeBlocks.forEach(codeBlock => {
        // Add language-specific classes if not already present
        if (!codeBlock.className) {
            const text = codeBlock.textContent;
            if (text.includes('use ai_utils') || text.includes('fn main')) {
                codeBlock.className = 'language-rust';
            } else if (text.includes('function') || text.includes('const')) {
                codeBlock.className = 'language-javascript';
            } else if (text.includes('import') || text.includes('export')) {
                codeBlock.className = 'language-typescript';
            }
        }
    });
}

// Add smooth scrolling for anchor links
function addSmoothScrolling() {
    const links = document.querySelectorAll('a[href^="#"]');
    
    links.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            
            const targetId = this.getAttribute('href');
            const targetElement = document.querySelector(targetId);
            
            if (targetElement) {
                targetElement.scrollIntoView({
                    behavior: 'smooth',
                    block: 'start'
                });
            }
        });
    });
}

// Add search functionality
function addSearchFunctionality() {
    const searchInput = document.querySelector('#search');
    if (!searchInput) return;
    
    searchInput.addEventListener('input', function() {
        const searchTerm = this.value.toLowerCase();
        const chapters = document.querySelectorAll('.chapter');
        
        chapters.forEach(chapter => {
            const text = chapter.textContent.toLowerCase();
            const isVisible = text.includes(searchTerm);
            chapter.style.display = isVisible ? 'block' : 'none';
        });
    });
}

// Add table of contents highlighting
function addTocHighlighting() {
    const tocLinks = document.querySelectorAll('.sidebar-nav a');
    const sections = document.querySelectorAll('h1, h2, h3');
    
    window.addEventListener('scroll', () => {
        let current = '';
        
        sections.forEach(section => {
            const sectionTop = section.offsetTop;
            const sectionHeight = section.clientHeight;
            
            if (window.pageYOffset >= sectionTop - 200) {
                current = section.getAttribute('id');
            }
        });
        
        tocLinks.forEach(link => {
            link.classList.remove('active');
            if (link.getAttribute('href') === `#${current}`) {
                link.classList.add('active');
            }
        });
    });
}

// Initialize additional features when DOM is ready
document.addEventListener('DOMContentLoaded', function() {
    addSearchFunctionality();
    addTocHighlighting();
});

// Export functions for potential external use
window.AIUtilsDocs = {
    addCopyButtonsToCodeBlocks,
    addSyntaxHighlighting,
    addSmoothScrolling,
    addSearchFunctionality,
    addTocHighlighting
}; 