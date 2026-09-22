document.querySelectorAll("[data-copy-citation]").forEach((button) => {
    button.hidden = false;
    button.addEventListener("click", async () => {
        const field = document.getElementById(button.dataset.copyCitation);
        const status = document.getElementById("citation-status");
        field.focus();
        field.select();
        if (!navigator.clipboard) {
            status.textContent = "Clipboard access is unavailable. The text is selected; use your browser's Copy command.";
            return;
        }
        try {
            await navigator.clipboard.writeText(field.value);
            status.textContent = "Copied to clipboard.";
        } catch (error) {
            console.warn("Could not copy citation:", error);
            status.textContent = "Could not access the clipboard. The text is selected; use your browser's Copy command.";
        }
    });
});
