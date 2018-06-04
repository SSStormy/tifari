import React, { Component } from 'react';
import './App.css';
import SearchField from './SearchField.js';

class App extends Component {

    constructor() {
        super();

        this.state = {
            images: []
        };

        this.onSearch = this.onSearch.bind(this);
        this.requeryImages = this.requeryImages.bind(this);
    }

    requeryImages(tagsArray) {
        try {
            fetch("http://localhost:8001/search", {
                method: "POST",

                body: JSON.stringify({
                    tags: tagsArray,
                    offset: 0,
                    max: 20
                })
            })
            .then(results => results.json())
            .then(data => {
                let mappedImages = data.results.map(img => {
                    return (
                        <li key={img.id}>
                            <p>{img.path}</p>
                        </li>
                    );
                });

                this.setState({images: mappedImages});
            });
        } catch(err) {
            console.error(err);
        }
    }

    onSearch(query) {
        let tags = query.split(" ");
        this.requeryImages(tags);
    }

    render() {
        return (
            <div>
            <SearchField onChange = {this.onSearch} />
            <ul>{this.state.images}</ul>
            </div>
        );
    }
}

export default App;
