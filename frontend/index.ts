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

document.addEventListener("change", async function (e) {
  if (!e.target) return;

  const formElement = (e.target as HTMLElement).closest(
    "[data-app='LiveForm']"
  );
  if (formElement) {
    const formData = new FormData(formElement as HTMLFormElement);
    const url = formElement.getAttribute("action")!;

    const searchParams = new URLSearchParams(formData as any);

    const fullUrl = `${url}?${searchParams.toString()}`;

    const resp = await fetch(fullUrl);

    const body = await resp.text();

    const targetSelector = formElement.getAttribute("data-target")!;
    const targetElement = formElement.querySelector(targetSelector);

    targetElement!.innerHTML = body;
  }
});
