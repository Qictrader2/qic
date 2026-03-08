/* elm-pkg-js
   (no ports — pure web component)
   Usage in Elm: Html.node "qr-svg" [ property "content" (Encode.string svgStr) ] []
*/

// Web component that renders raw SVG via innerHTML.
// Elm's virtual DOM can't reliably set innerHTML directly,
// but a custom element property setter works perfectly.
class QrSvg extends HTMLElement {
    set content(val) {
        this._content = val;
        if (this.isConnected) this._render();
    }
    get content() { return this._content || ''; }
    connectedCallback() {
        if (this._content) this._render();
    }
    _render() {
        this.innerHTML = this._content || '';
        // Make the SVG fill its container
        var svg = this.querySelector('svg');
        if (svg) {
            svg.style.width = '100%';
            svg.style.height = '100%';
            svg.style.display = 'block';
        }
    }
}
customElements.define('qr-svg', QrSvg);
