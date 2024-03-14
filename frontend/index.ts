document.addEventListener("click", function (e) {
  if (!e.target) return;

  const clickedElement = (e.target as HTMLElement).closest(
    "[data-app='ToggleClass']"
  );
  if (clickedElement) {
    const className = clickedElement.getAttribute("data-class")!;
    const targetSelector = clickedElement.getAttribute("data-target")!;

    const targetElement =
      clickedElement.closest(targetSelector) ||
      document.querySelector(targetSelector);

    if (targetElement) {
      targetElement.classList.toggle(className);
    }
  }
});
