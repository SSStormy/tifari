import React, { Component } from 'react';
import './App.css';

import SearchField from './SearchField.js';
import ImageSlot from './ImageSlot.js';
import ImageEditor from './ImageEditor.js'
import TifariAPI from "./APIComms.js"

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
        this.escKeyListener = this.escKeyListener.bind(this);
        this.viewToBeTaggedList = this.viewToBeTaggedList.bind(this);
        this.mapSelectedImages = this.mapSelectedImages.bind(this);
    }

    hideEditorSidebar() {
        this.setState({
            selectedImage: null
        });
    }

    escKeyListener(ev) {
        if(ev.key === "Escape") {
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

    mapSelectedImages(images) {
        let mappedImages = images.results.map(img => 
            <ImageSlot img={img} key={img.id} onClick={() => this.onSelectImage(img)}/>);
        this.setState({queriedImages: mappedImages});
    }

    requeryImages(tagsArray) {
        TifariAPI.search(tagsArray).then(this.mapSelectedImages);
    }

    viewToBeTaggedList() {
        TifariAPI.getToBeTaggedList().then(this.mapSelectedImages);
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
                    <button onClick={this.viewToBeTaggedList}>View To-Tag List</button>
                    <SearchField onChange = {this.onSearch} />
                </header>

                <ul>{this.state.queriedImages}</ul>
            </div>
        );
    }
}

export default App;
