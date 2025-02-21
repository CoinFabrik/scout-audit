document.addEventListener("DOMContentLoaded", () => {
  const severityButtons = document.querySelectorAll(".severity-filter");
  let activeSeverities = new Set([
    "critical",
    "medium",
    "minor",
    "enhancement",
  ]);

  function updateVisibility() {
    const activeCategory =
      document
        .querySelector(".category.active")
        ?.getAttribute("data-category") || "all";

    document.querySelectorAll(".vulnerability").forEach((vulnerability) => {
      const severityClass = Array.from(
        vulnerability.querySelector(".icon i").classList
      ).find((cls) =>
        ["critical", "medium", "minor", "enhancement"].includes(cls)
      );
      const categorySection = vulnerability.closest(".category-section");
      const categoryName = categorySection.getAttribute("data-category");

      const matchesSeverity = activeSeverities.has(severityClass);
      const matchesCategory =
        activeCategory === "all" || categoryName === activeCategory;

      vulnerability.classList.toggle(
        "hidden",
        !matchesSeverity || !matchesCategory
      );
    });

    severityButtons.forEach((button) => {
      const severity = button.getAttribute("data-severity");
      button.classList.toggle("active", activeSeverities.has(severity));
    });
  }

  severityButtons.forEach((button) => {
    const severity = button.getAttribute("data-severity");
    if (activeSeverities.has(severity)) {
      button.classList.add("active");
    }

    button.addEventListener("click", () => {
      const severity = button.getAttribute("data-severity");
      if (activeSeverities.has(severity)) {
        activeSeverities.delete(severity);
      } else {
        activeSeverities.add(severity);
      }
      updateVisibility();
    });
  });

  updateVisibility();
});
