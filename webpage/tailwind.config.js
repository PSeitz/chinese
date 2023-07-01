/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.{html,js,rs}"],
  theme: {
    extend: {},
    container: {
      //center: true,
      padding: '2rem',
    },
  },
  daisyui: {
      themes: ["light", "dark", "cupcake", "bumblebee", "emerald", "corporate", "synthwave", "retro", "cyberpunk", "valentine", "halloween", "garden", "forest", "aqua", "lofi", "pastel", "fantasy", "wireframe", "black", "luxury", "dracula", "cmyk", "autumn", "business", "acid", "lemonade", "night", "coffee", "winter"],
      //themes: [
        //{
          //mytheme: {
            //"primary": "#004aad",
            //"secondary": "#545454",
            //"accent": "#37CDBE",
            //"neutral": "#3D4451",
            //"base-100": "#FFFFFF",
            //"info": "#3ABFF8",
            //"success": "#36D399",
            //"warning": "#FBBD23",
            //"error": "#F87272",
          //},
        //},
      //],
    },
  plugins: [require("daisyui")],
}
