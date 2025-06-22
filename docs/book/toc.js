// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="index.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="getting-started/installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="getting-started/quick-start.html"><strong aria-hidden="true">2.</strong> Quick Start</a></li><li class="chapter-item expanded "><a href="getting-started/configuration.html"><strong aria-hidden="true">3.</strong> Configuration</a></li><li class="chapter-item expanded affix "><li class="part-title">Core Concepts</li><li class="chapter-item expanded "><a href="core-concepts/architecture.html"><strong aria-hidden="true">4.</strong> Architecture Overview</a></li><li class="chapter-item expanded "><a href="core-concepts/async-operations.html"><strong aria-hidden="true">5.</strong> Async Operations</a></li><li class="chapter-item expanded "><a href="core-concepts/error-handling.html"><strong aria-hidden="true">6.</strong> Error Handling</a></li><li class="chapter-item expanded affix "><li class="part-title">Modules</li><li class="chapter-item expanded "><a href="modules/openai/overview.html"><strong aria-hidden="true">7.</strong> Overview</a></li><li class="chapter-item expanded "><a href="modules/openai/chat-completions.html"><strong aria-hidden="true">8.</strong> Chat Completions</a></li><li class="chapter-item expanded "><a href="modules/openai/embeddings.html"><strong aria-hidden="true">9.</strong> Embeddings</a></li><li class="chapter-item expanded "><a href="modules/openai/image-generation.html"><strong aria-hidden="true">10.</strong> Image Generation</a></li><li class="chapter-item expanded "><a href="modules/openai/audio-transcription.html"><strong aria-hidden="true">11.</strong> Audio Transcription</a></li><li class="chapter-item expanded "><a href="modules/qdrant/overview.html"><strong aria-hidden="true">12.</strong> Overview</a></li><li class="chapter-item expanded "><a href="modules/qdrant/collections.html"><strong aria-hidden="true">13.</strong> Collections</a></li><li class="chapter-item expanded "><a href="modules/qdrant/vector-search.html"><strong aria-hidden="true">14.</strong> Vector Search</a></li><li class="chapter-item expanded "><a href="modules/text-splitter/overview.html"><strong aria-hidden="true">15.</strong> Overview</a></li><li class="chapter-item expanded "><a href="modules/text-splitter/chunking.html"><strong aria-hidden="true">16.</strong> Chunking</a></li><li class="chapter-item expanded "><a href="modules/text-splitter/tokenization.html"><strong aria-hidden="true">17.</strong> Tokenization</a></li><li class="chapter-item expanded "><a href="modules/langfuse/overview.html"><strong aria-hidden="true">18.</strong> Overview</a></li><li class="chapter-item expanded "><a href="modules/langfuse/monitoring.html"><strong aria-hidden="true">19.</strong> Monitoring</a></li><li class="chapter-item expanded "><a href="modules/langfuse/traces-spans.html"><strong aria-hidden="true">20.</strong> Traces &amp; Spans</a></li><li class="chapter-item expanded "><a href="modules/common/overview.html"><strong aria-hidden="true">21.</strong> Overview</a></li><li class="chapter-item expanded "><a href="modules/common/base64.html"><strong aria-hidden="true">22.</strong> Base64</a></li><li class="chapter-item expanded "><a href="modules/common/image-processing.html"><strong aria-hidden="true">23.</strong> Image Processing</a></li><li class="chapter-item expanded affix "><li class="part-title">Examples</li><li class="chapter-item expanded "><a href="examples/basic-chat-bot.html"><strong aria-hidden="true">24.</strong> Basic Chat Bot</a></li><li class="chapter-item expanded "><a href="examples/document-qa.html"><strong aria-hidden="true">25.</strong> Document Q&amp;A</a></li><li class="chapter-item expanded "><a href="examples/image-analysis.html"><strong aria-hidden="true">26.</strong> Image Analysis</a></li><li class="chapter-item expanded "><a href="examples/multimodal-agent.html"><strong aria-hidden="true">27.</strong> Multimodal Agent</a></li><li class="chapter-item expanded affix "><li class="part-title">API Reference</li><li class="chapter-item expanded "><a href="api/openai-service.html"><strong aria-hidden="true">28.</strong> OpenAI Service</a></li><li class="chapter-item expanded "><a href="api/qdrant-service.html"><strong aria-hidden="true">29.</strong> Qdrant Service</a></li><li class="chapter-item expanded "><a href="api/text-splitter.html"><strong aria-hidden="true">30.</strong> Text Splitter</a></li><li class="chapter-item expanded "><a href="api/langfuse-service.html"><strong aria-hidden="true">31.</strong> Langfuse Service</a></li><li class="chapter-item expanded "><a href="api/common-types.html"><strong aria-hidden="true">32.</strong> Common Types</a></li><li class="chapter-item expanded affix "><li class="part-title">Deployment</li><li class="chapter-item expanded "><a href="deployment/environment-variables.html"><strong aria-hidden="true">33.</strong> Environment Variables</a></li><li class="chapter-item expanded "><a href="deployment/production-setup.html"><strong aria-hidden="true">34.</strong> Production Setup</a></li><li class="chapter-item expanded "><a href="deployment/performance-tuning.html"><strong aria-hidden="true">35.</strong> Performance Tuning</a></li><li class="chapter-item expanded affix "><li class="part-title">Contributing</li><li class="chapter-item expanded "><a href="contributing/development-setup.html"><strong aria-hidden="true">36.</strong> Development Setup</a></li><li class="chapter-item expanded "><a href="contributing/code-style.html"><strong aria-hidden="true">37.</strong> Code Style</a></li><li class="chapter-item expanded "><a href="contributing/testing.html"><strong aria-hidden="true">38.</strong> Testing</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
