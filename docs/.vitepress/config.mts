import { defineConfig } from "vitepress";

export default defineConfig({
  base: "/crafting-rust",
  title: "Crafting Rust",
  description:
    "From First Principles to a Production-Ready Engine — A hands-on, practical Rust book",
  lastUpdated: false,
  cleanUrls: true,

  locales: {
    root: {
      label: "English",
      lang: "en",
      link: "/en/",
      themeConfig: {
        nav: [
          { text: "Home", link: "/en/" },
          { text: "Roadmap", link: "/en/roadmap" },
          { text: "Outline", link: "/en/outline" },
          { text: "Chapters", link: "/en/chapters/01-why-rust" },
        ],
        sidebar: [
          {
            text: "Part 0 — Starting the Journey",
            collapsed: false,
            items: [
              {
                text: "1. Why Rust? How We'll Learn Rust",
                link: "/en/chapters/01-why-rust",
              },
              {
                text: "2. Getting Started with Rust",
                link: "/en/chapters/02-getting-started",
              },
              {
                text: "3. Rust's Basic Building Blocks",
                link: "/en/chapters/03-basic-building-blocks",
              },
              { text: "4. Control Flow", link: "/en/chapters/04-control-flow" },
              {
                text: "5. Structs: Modeling Real Things",
                link: "/en/chapters/05-structs",
              },
            ],
          },
          {
            text: "Part I — Ownership",
            collapsed: false,
            items: [
              {
                text: "6. Ownership",
                link: "/en/chapters/06-ownership",
              },
              {
                text: "7. Borrowing and References",
                link: "/en/chapters/07-borrowing-and-references",
              },
              {
                text: "8. Strings, Slices and Ownership in Practice",
                link: "/en/chapters/08-strings-slices-and-ownership",
              },
              {
                text: "9. Collections",
                link: "/en/chapters/09-collections",
              },
            ],
          },
          {
            text: "Part II — Modeling Business Logic",
            collapsed: false,
            items: [
              {
                text: "10. Enums and Pattern Matching",
                link: "/en/chapters/10-enums-and-pattern-matching",
              },
              {
                text: "11. Option",
                link: "/en/chapters/11-option",
              },
              {
                text: "12. Result and Error Handling",
                link: "/en/chapters/12-result-and-error-handling",
              },
              {
                text: "13. Modules and Project Structure",
                link: "/en/chapters/13-modules-and-project-structure",
              },
            ],
          },
          {
            text: "Part III — Making Rust Code Powerful",
            collapsed: false,
            items: [
              {
                text: "14. Generics",
                link: "/en/chapters/14-generics",
              },
              {
                text: "15. Traits",
                link: "/en/chapters/15-traits",
              },
              {
                text: "16. Lifetimes",
                link: "/en/chapters/16-lifetimes",
              },
              {
                text: "17. Iterators",
                link: "/en/chapters/17-iterators",
              },
              {
                text: "18. Closures",
                link: "/en/chapters/18-closures",
              },
              {
                text: "19. Smart Pointers",
                link: "/en/chapters/19-smart-pointers",
              },
            ],
          },
          {
            text: "Part IV — Concurrency & Async",
            collapsed: false,
            items: [
              {
                text: "20. Concurrency",
                link: "/en/chapters/20-concurrency",
              },
              {
                text: "21. Async Rust",
                link: "/en/chapters/21-async-rust",
              },
            ],
          },
          {
            text: "Part V — Application Layer",
            collapsed: false,
            items: [
              {
                text: "22. Persistence in Rust",
                link: "/en/chapters/22-persistence",
              },
              {
                text: "23. Building a Rust Web API",
                link: "/en/chapters/23-web-api",
              },
              {
                text: "24. Production Rust",
                link: "/en/chapters/24-production-rust",
              },
            ],
          },
          {
            text: "Part VI — Advanced Rust & Metaprogramming",
            collapsed: false,
            items: [
              {
                text: "25. Declarative Macros",
                link: "/en/chapters/25-declarative-macros",
              },
              {
                text: "26. Unsafe Rust & Safe Abstractions",
                link: "/en/chapters/26-unsafe-rust",
              },
              {
                text: "27. Conclusion & Beyond",
                link: "/en/chapters/27-conclusion-and-beyond",
              },
            ],
          },
        ],
        outline: {
          level: [2, 3],
          label: "On this page",
        },
        socialLinks: [
          {
            icon: "github",
            link: "https://github.com/rimonmath/crafting-rust",
          },
        ],
      },
    },
    bn: {
      label: "বাংলা",
      lang: "bn",
      link: "/bn/",
      themeConfig: {
        nav: [
          { text: "হোম", link: "/bn/" },
          { text: "রোডম্যাপ", link: "/bn/roadmap" },
          { text: "আউটলাইন", link: "/bn/outline" },
          { text: "অধ্যায়সমূহ", link: "/bn/chapters/01-why-rust" },
        ],
        sidebar: [
          {
            text: "পার্ট ০ — যাত্রা শুরু",
            collapsed: false,
            items: [
              {
                text: "১. কেন রাস্ট? আমরা কীভাবে রাস্ট শিখব",
                link: "/bn/chapters/01-why-rust",
              },
              {
                text: "২. রাস্ট দিয়ে শুরু করা",
                link: "/bn/chapters/02-getting-started",
              },
              {
                text: "৩. রাস্টের মৌলিক ভিত্তি",
                link: "/bn/chapters/03-basic-building-blocks",
              },
              {
                text: "৪. কন্ট্রোল ফ্লো",
                link: "/bn/chapters/04-control-flow",
              },
              {
                text: "৫. স্ট্রাক্ট: বাস্তব জিনিসের মডেলিং",
                link: "/bn/chapters/05-structs",
              },
            ],
          },
          {
            text: "পার্ট ১ — ওনারশিপ",
            collapsed: false,
            items: [
              {
                text: "৬. ওনারশিপ",
                link: "/bn/chapters/06-ownership",
              },
              {
                text: "৭. বরোয়িং এবং রেফারেন্স",
                link: "/bn/chapters/07-borrowing-and-references",
              },
              {
                text: "৮. স্ট্রিং, স্লাইস এবং ওনারশিপের ব্যবহারিক প্রয়োগ",
                link: "/bn/chapters/08-strings-slices-and-ownership",
              },
              {
                text: "৯. কালেকশনস",
                link: "/bn/chapters/09-collections",
              },
            ],
          },
          {
            text: "পার্ট ২ — বিজনেস লজিক মডেলিং",
            collapsed: false,
            items: [
              {
                text: "১০. এনাম এবং প্যাটার্ন ম্যাচিং",
                link: "/bn/chapters/10-enums-and-pattern-matching",
              },
              {
                text: "১১. অপশন (Option)",
                link: "/bn/chapters/11-option",
              },
              {
                text: "১২. রেজাল্ট এবং এরর হ্যান্ডলিং (Result)",
                link: "/bn/chapters/12-result-and-error-handling",
              },
              {
                text: "১৩. মডিউল এবং প্রজেক্ট স্ট্রাকচার",
                link: "/bn/chapters/13-modules-and-project-structure",
              },
            ],
          },
          {
            text: "পার্ট ৩ — রাস্ট কোডকে শক্তিশালী করা",
            collapsed: false,
            items: [
              {
                text: "১৪. জেনেরিকস (Generics)",
                link: "/bn/chapters/14-generics",
              },
              {
                text: "১৫. ট্রেইটস (Traits)",
                link: "/bn/chapters/15-traits",
              },
              {
                text: "১৬. লাইফটাইমস (Lifetimes)",
                link: "/bn/chapters/16-lifetimes",
              },
              {
                text: "১৭. ইটারেটরস (Iterators)",
                link: "/bn/chapters/17-iterators",
              },
              {
                text: "১৮. ক্লোজারস (Closures)",
                link: "/bn/chapters/18-closures",
              },
              {
                text: "১৯. স্মার্ট পয়েন্টারস (Smart Pointers)",
                link: "/bn/chapters/19-smart-pointers",
              },
            ],
          },
          {
            text: "পার্ট ৪ — কনকারেন্সি ও অ্যাসিঙ্ক (Concurrency & Async)",
            collapsed: false,
            items: [
              {
                text: "২০. কনকারেন্সি (Concurrency)",
                link: "/bn/chapters/20-concurrency",
              },
              {
                text: "২১. অ্যাসিঙ্ক রাস্ট (Async Rust)",
                link: "/bn/chapters/21-async-rust",
              },
            ],
          },
          {
            text: "পার্ট ৫ — অ্যাপ্লিকেশন লেয়ার (Application Layer)",
            collapsed: false,
            items: [
              {
                text: "২২. পারসিস্টেন্স (Persistence in Rust)",
                link: "/bn/chapters/22-persistence",
              },
              {
                text: "২৩. ওয়েব এপিআই (Building a Rust Web API)",
                link: "/bn/chapters/23-web-api",
              },
              {
                text: "২৪. প্রোডাকশন রাস্ট (Production Rust)",
                link: "/bn/chapters/24-production-rust",
              },
            ],
          },
          {
            text: "পার্ট ৬ — অ্যাডভান্সড রাস্ট ও মেটাপ্রোগ্রামিং (Advanced Rust)",
            collapsed: false,
            items: [
              {
                text: "২৫. ডিক্লারেটিভ ম্যাক্রো (Declarative Macros)",
                link: "/bn/chapters/25-declarative-macros",
              },
              {
                text: "২৬. আনসেফ রাস্ট ও নিরাপদ অ্যাবস্ট্রাকশন (Unsafe Rust)",
                link: "/bn/chapters/26-unsafe-rust",
              },
              {
                text: "২৭. সমাপ্তি ও পরবর্তী পথচলা (Conclusion & Beyond)",
                link: "/bn/chapters/27-conclusion-and-beyond",
              },
            ],
          },
        ],
        outline: {
          level: [2, 3],
          label: "এই পৃষ্ঠায়",
        },
        socialLinks: [
          {
            icon: "github",
            link: "https://github.com/rimonmath/crafting-rust",
          },
        ],
      },
    },
  },
});
