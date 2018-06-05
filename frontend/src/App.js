import React, { Component } from 'react';
import './App.css';

import SearchField from './SearchField.js';
import ImageSlot from './ImageSlot.js';
import Constants from './Config.js'
import ImageEditor from './ImageEditor.js'

class App extends Component {

    constructor(props) {
        super(props);

        this.state = {
            queriedImages: [],
            selectedImage: null,
        };

        this.onSearch = this.onSearch.bind(this);
        this.requeryImages = this.requeryImages.bind(this);
        this.onSelectImage= this.onSelectImage.bind(this);
        this.hideEditorSidebar= this.hideEditorSidebar.bind(this);
        this.escKeyListener= this.escKeyListener.bind(this);
    }

    hideEditorSidebar() {
        this.setState({
            selectedImage: null
        });
    }

    escKeyListener(ev) {
        if(ev.keyCode === 27) {
            this.hideEditorSidebar();
        }
    }

    componentDidMount() {
        document.addEventListener("keypress", this.escKeyListener, false);
    }

    componentWillUnMount() {
        document.removeEventListener("keypress", this.escKeyListener, false);
    }

    onSelectImage(img) {
        this.setState({
            selectedImage: img
        });
    }

    requeryImages(tagsArray) {
        try {
            fetch(Constants.ENDPOINT_API_SEARCH, {
                method: "POST",

                body: JSON.stringify({
                    tags: tagsArray,
                    offset: 0,
                    max: 20
                })
            })
            .then(results => results.json())
            .then(data => {
                let mappedImages = data.results.map(img => 
                    <ImageSlot img={img} key={img.id} onClick={() => this.onSelectImage(img)}/>);
                this.setState({queriedImages: mappedImages});
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
            <div className="App">
                {this.state.selectedImage !== null &&
                    <ImageEditor img={this.state.selectedImage}/>
                }

                <header>
                    <SearchField onChange = {this.onSearch} />
                </header>

                <ul>{this.state.queriedImages}</ul>
            </div>
        );
    }
}

export default App;
