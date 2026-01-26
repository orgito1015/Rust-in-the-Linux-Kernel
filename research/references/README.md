# References and Bibliography

This directory stores bibliographic information, link collections, and reference materials about Rust in the Linux Kernel.

## Purpose

Use this space for:
- **Academic papers** (links or citations, not full PDFs unless open access)
- **Blog posts** and articles
- **Conference talks** (links to videos and slides)
- **Technical specifications**
- **Curated link collections**
- **Bibliography entries**

## What NOT to Store

⚠️ **Do not commit copyrighted files** without permission:
- Academic papers behind paywalls (link to them instead)
- Copyrighted books or chapters
- Conference proceedings without authorization

✅ **Safe to include**:
- Links to papers and resources
- Open access publications
- Your own summaries and notes
- BibTeX or other citation formats

## Organization

Suggested structure:
```
references/
├── papers.md              # Academic paper links and citations
├── blogs.md               # Blog posts and articles
├── talks.md               # Conference presentations
├── books.md               # Book references
├── links.md               # General web resources
└── bibliography.bib       # BibTeX entries (optional)
```

## Reference Format

### Academic Papers

```markdown
## Paper Title (Year)

**Authors**: Name1, Name2, Name3

**Venue**: Conference/Journal Name

**Link**: [URL if available]

**Abstract**: Brief summary

**Key Takeaways**:
- Point 1
- Point 2

**Relevance to Project**: Why this matters for Rust-for-Linux
```

### Blog Posts and Articles

```markdown
## Article Title

**Author**: Name

**Date**: YYYY-MM-DD

**URL**: [Link]

**Summary**: Brief description

**Key Points**:
- Important insight 1
- Important insight 2
```

### Conference Talks

```markdown
## Talk Title

**Speaker**: Name

**Conference**: Conference Name (Year)

**Video**: [Link to recording]

**Slides**: [Link if available]

**Summary**: What was presented

**Highlights**:
- Key point 1
- Key point 2
```

## Maintaining References

1. **Keep links alive**: Periodically check for broken links
2. **Use archives**: Consider linking to archive.org for important resources
3. **Track access date**: Note when you accessed paywalled content
4. **Version URLs**: Some resources update; note version if important
5. **Organize by topic**: Group related references together

## BibTeX Example

If using BibTeX for academic citations:

```bibtex
@article{author2023rust,
  title={Rust in the Linux Kernel: A Case Study},
  author={Author, First and Author, Second},
  journal={Journal Name},
  year={2023},
  volume={42},
  pages={1--20},
  url={https://example.com/paper}
}

@inproceedings{presenter2023talk,
  title={Writing Safe Linux Drivers in Rust},
  author={Presenter, Name},
  booktitle={Linux Plumbers Conference},
  year={2023},
  url={https://example.com/talk}
}
```

## Citation Management

Consider using:
- **Zotero**: Free, open-source reference manager
- **Mendeley**: Academic reference manager
- **BibTeX**: For LaTeX-based papers
- **Markdown files**: Simple, version-control friendly

## Sharing References

When adding references:
1. Ensure links are accessible
2. Provide enough context
3. Explain relevance to the project
4. Check for duplicates before adding
5. Keep formatting consistent

## Copyright and Fair Use

**Guidelines**:
- ✅ Link to copyrighted material
- ✅ Quote brief excerpts (with attribution)
- ✅ Summarize in your own words
- ❌ Copy entire articles or papers
- ❌ Distribute paywalled content
- ❌ Violate copyright terms

## Contributing

When adding references:
1. Use the suggested format
2. Verify links work
3. Add meaningful summaries
4. Tag by topic if possible
5. Keep information up-to-date

---

**Good references make good research!** Help build a comprehensive knowledge base.
