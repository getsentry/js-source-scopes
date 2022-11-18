function Foo() {}
Foo.prototype.bar = () => {};
export default (new Foo).bar();