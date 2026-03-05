<script>
    export let data;
    // Если вдруг придет null или undefined, ставим белый
    if (!data) data = "#ffffff";
</script>

<div class="color-container">
    <!-- Сам пикер цвета -->
    <input type="color" bind:value={data}>
    <!-- Текстовое поле для копирования/вставки hex -->
    <input type="text" bind:value={data} maxlength="7">
</div>

<style>
    .color-container {
        display: flex;
        align-items: center;
        gap: 5px;
        height: 100%;
        padding: 2px;
    }

    input[type="color"] {
        border: none;
        width: 30px;
        height: 25px;
        padding: 0;
        background: none;
        cursor: pointer;
    }

    input[type="text"] {
        width: 60px;
        border: none;
        background-color: transparent;
        color: #f0f0f0;
        font-family: monospace;
        font-size: 12px;
    }
</style>