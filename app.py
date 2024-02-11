from flask import Flask, render_template, request

app = Flask(__name__)


@app.route('/')
def index():
    return render_template("index.html", title="Home",
                           scripts=["https://cdnjs.cloudflare.com/ajax/libs/p5.js/0.9.0/p5.min.js",
                                    "https://cdnjs.cloudflare.com/ajax/libs/p5.js/0.9.0/addons/p5.dom.min.js",
                                    "https://unpkg.com/ml5@latest/dist/ml5.min.js"]
                           )

@app.route('/api/pos')
def api_pos():
    return "200"

if __name__ == '__main__':
    app.run(debug=True)
