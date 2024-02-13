from flask import Flask, render_template, request

app = Flask(__name__)

isRight = False

@app.route('/')
def index():
    return render_template("index.html", title="Home",
                           scripts=["https://cdnjs.cloudflare.com/ajax/libs/p5.js/0.9.0/p5.min.js",
                                    "https://cdnjs.cloudflare.com/ajax/libs/p5.js/0.9.0/addons/p5.dom.min.js",
                                    "https://unpkg.com/ml5@latest/dist/ml5.min.js"]
                           )


@app.route('/about')
def about():
    return render_template("about.html", title="About", scripts=[])

@app.route('/api/pos', methods=['GET'])
def api_pos():
    return str(isRight)


@app.route('/api/setpos', methods=['POST'])
def process():
    data = request.json['data']
    global isRight
    isRight = bool(data)
    return "hi"


if __name__ == '__main__':
    app.run(debug=True)
